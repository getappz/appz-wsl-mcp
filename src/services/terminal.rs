use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tracing::{debug, error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmuxResult {
    pub success: bool,
    pub output: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSession {
    pub name: String,
    pub created: String,
}

pub struct TerminalService;

impl TerminalService {
    pub fn new() -> Self {
        Self
    }

    async fn exec_tmux(distro: &str, args: &[&str]) -> TmuxResult {
        let mut cmd_args = vec!["-d", distro, "tmux"];
        cmd_args.extend_from_slice(args);

        let output = Command::new("wsl.exe")
            .args(&cmd_args)
            .output()
            .await;

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                let success = out.status.success();
                debug!("tmux {args:?} → exit={} out={} err={}", out.status, stdout.len(), stderr.len());
                TmuxResult {
                    success,
                    output: if success { stdout } else { stderr },
                    exit_code: out.status.code().unwrap_or(-1),
                }
            }
            Err(e) => {
                error!("tmux exec failed: {e}");
                TmuxResult {
                    success: false,
                    output: e.to_string(),
                    exit_code: -1,
                }
            }
        }
    }

    pub async fn create_session(&self, distro: &str, name: &str) -> TmuxResult {
        info!("Creating tmux session {name} on {distro}");
        let name = if name.is_empty() { "wsl-mcp" } else { name };
        Self::exec_tmux(distro, &["new-session", "-d", "-s", name]).await
    }

    pub async fn send_keys(&self, distro: &str, session: &str, input: &str) -> TmuxResult {
        info!("Sending keys to {session}: {input}");
        Self::exec_tmux(distro, &["send-keys", "-t", session, "-l", input]).await
    }

    pub async fn send_enter(&self, distro: &str, session: &str) -> TmuxResult {
        debug!("Send Enter to {session}");
        Self::exec_tmux(distro, &["send-keys", "-t", session, "Enter"]).await
    }

    pub async fn read_output(&self, distro: &str, session: &str, lines: Option<usize>) -> TmuxResult {
        let lines = lines.unwrap_or(100);
        let scroll = format!("-{}", lines);
        debug!("Reading {lines} lines from {session}");
        Self::exec_tmux(distro, &["capture-pane", "-t", session, "-p", "-S", &scroll, "-E", "-"]).await
    }

    pub async fn list_sessions(&self, distro: &str) -> Vec<TerminalSession> {
        info!("Listing tmux sessions on {distro}");
        let result = Self::exec_tmux(distro, &["list-sessions", "-F", "#{session_name}:#{session_created}"]).await;
        if !result.success {
            return vec![];
        }
        result.output.lines().filter_map(|line| {
            let line = line.trim();
            if line.is_empty() { return None; }
            let mut parts = line.splitn(2, ':');
            let name = parts.next()?.to_string();
            let created = parts.next().unwrap_or("").to_string();
            Some(TerminalSession { name, created })
        }).collect()
    }

    pub async fn kill_session(&self, distro: &str, session: &str) -> TmuxResult {
        info!("Killing tmux session {session} on {distro}");
        Self::exec_tmux(distro, &["kill-session", "-t", session]).await
    }

    pub async fn has_session(&self, distro: &str, session: &str) -> bool {
        let result = Self::exec_tmux(distro, &["has-session", "-t", session]).await;
        result.success
    }

    pub async fn wait_for_output(
        &self,
        distro: &str,
        session: &str,
        marker: &str,
        max_polls: u32,
        poll_ms: u64,
    ) -> TmuxResult {
        info!("Waiting for marker '{marker}' in {session} (max {max_polls}×{poll_ms}ms)");
        let poll_dur = std::time::Duration::from_millis(poll_ms);
        let scroll = format!("-{}", 500);
        for i in 0..max_polls {
            let result = Self::exec_tmux(distro, &["capture-pane", "-t", session, "-p", "-S", &scroll, "-E", "-"]).await;
            if result.success && result.output.contains(marker) {
                return result;
            }
            if i == max_polls - 1 {
                return TmuxResult {
                    success: false,
                    output: format!("Timeout: marker '{marker}' not found after {max_polls} polls"),
                    exit_code: -1,
                };
            }
            tokio::time::sleep(poll_dur).await;
        }
        TmuxResult { success: false, output: "No polls configured".into(), exit_code: -1 }
    }

    pub async fn clear_scrollback(&self, distro: &str, session: &str) -> TmuxResult {
        debug!("Clearing scrollback for {session}");
        Self::exec_tmux(distro, &["clear-history", "-t", session]).await
    }
}
