use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::process::Command;
use tracing::{debug, error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WslDistribution {
    pub name: String,
    pub is_default: bool,
    pub state: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WslInfo {
    pub version: String,
    pub status: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub output: String,
    pub exit_code: i32,
}

pub struct WslService;

impl WslService {
    pub fn new() -> Self {
        Self
    }

    async fn exec_wsl(args: &[&str]) -> CommandResult {
        let output = Command::new("wsl.exe")
            .args(args)
            .output()
            .await;

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                let success = out.status.success();
                debug!("wsl {args:?} → exit={} out={} err={}", out.status, stdout.len(), stderr.len());
                CommandResult {
                    success,
                    output: if success { stdout } else { stderr },
                    exit_code: out.status.code().unwrap_or(-1),
                }
            }
            Err(e) => {
                error!("wsl exec failed: {e}");
                CommandResult {
                    success: false,
                    output: e.to_string(),
                    exit_code: -1,
                }
            }
        }
    }

    pub async fn list_distributions(&self) -> Vec<WslDistribution> {
        info!("Listing WSL distributions");
        let result = Self::exec_wsl(&["--list", "--verbose"]).await;
        let mut distros = Vec::new();

        for line in result.output.lines().skip(1) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let name = parts[0].trim_start_matches('*').trim().to_string();
                distros.push(WslDistribution {
                    is_default: line.contains('*'),
                    state: parts[1].to_string(),
                    version: parts[2].to_string(),
                    name,
                });
            }
        }
        distros
    }

    pub async fn get_info(&self) -> WslInfo {
        info!("Getting WSL system info");
        let ver = Self::exec_wsl(&["--version"]).await;
        let status = Self::exec_wsl(&["--status"]).await;
        WslInfo {
            version: ver.output,
            status: status.output,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub async fn install_distro(&self, distro: &str) -> CommandResult {
        info!("Installing WSL distro: {distro}");
        Self::exec_wsl(&["--install", distro]).await
    }

    pub async fn start_distro(&self, distro: &str) -> CommandResult {
        info!("Starting WSL distro: {distro}");
        Self::exec_wsl(&["-d", distro, "--exec", "true"]).await
    }

    pub async fn stop_distro(&self, distro: &str) -> CommandResult {
        info!("Stopping WSL distro: {distro}");
        Self::exec_wsl(&["--terminate", distro]).await
    }

    pub async fn shutdown(&self) -> CommandResult {
        info!("Shutting down all WSL");
        Self::exec_wsl(&["--shutdown"]).await
    }

    pub async fn execute_command(&self, distro: &str, command: &str) -> CommandResult {
        info!("Executing in {distro}: {command}");
        if distro.is_empty() {
            Self::exec_wsl(&["--exec", command]).await
        } else {
            Self::exec_wsl(&["-d", distro, "--exec", command]).await
        }
    }

    pub async fn configure(&self, settings: &HashMap<String, String>) -> CommandResult {
        info!("Configuring WSL");
        let profile = std::env::var("USERPROFILE").unwrap_or_else(|_| r"C:\Users\Default".into());
        let wslconfig_path = format!(r"{}\.wslconfig", profile);

        let mut content = String::from("[wsl2]\n");
        for (k, v) in settings {
            content.push_str(&format!("{k}={v}\n"));
        }

        match tokio::fs::write(&wslconfig_path, &content).await {
            Ok(_) => {
                self.shutdown().await;
                CommandResult {
                    success: true,
                    output: format!("Written to {wslconfig_path}. WSL restarted."),
                    exit_code: 0,
                }
            }
            Err(e) => CommandResult {
                success: false,
                output: e.to_string(),
                exit_code: -1,
            },
        }
    }

    pub async fn export_distro(&self, distro: &str, output_path: &str) -> CommandResult {
        info!("Exporting {distro} → {output_path}");
        Self::exec_wsl(&["--export", distro, output_path]).await
    }

    pub async fn import_distro(&self, distro: &str, tar_path: &str, install_path: &str) -> CommandResult {
        info!("Importing {distro} from {tar_path} → {install_path}");
        Self::exec_wsl(&["--import", distro, install_path, tar_path]).await
    }

    pub async fn copy_from_wsl(&self, distro: &str, wsl_path: &str, windows_path: &str) -> CommandResult {
        info!("Copy {wsl_path} (WSL) → {windows_path} (Windows)");
        let conv = Self::exec_wsl(&["-d", distro, "wslpath", "-w", wsl_path]).await;
        if !conv.success {
            return conv;
        }
        let source = conv.output.trim();
        match tokio::fs::copy(source, windows_path).await {
            Ok(_) => CommandResult { success: true, output: "Copy OK".into(), exit_code: 0 },
            Err(e) => CommandResult { success: false, output: e.to_string(), exit_code: -1 },
        }
    }

    pub async fn copy_to_wsl(&self, distro: &str, windows_path: &str, wsl_path: &str) -> CommandResult {
        info!("Copy {windows_path} (Windows) → {wsl_path} (WSL)");
        let win_escaped = windows_path.replace('\\', "/");
        self.execute_command(distro, &format!("cp -r '{win_escaped}' '{wsl_path}'")).await
    }
}
