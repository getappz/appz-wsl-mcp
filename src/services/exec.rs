use crate::config::AppConfig;
use regex::Regex;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tracing::{debug, info, warn};
use uuid::Uuid;

pub struct ExecService {
    cfg: Arc<AppConfig>,
}

impl ExecService {
    pub fn new(cfg: Arc<AppConfig>) -> Self {
        Self { cfg }
    }

    pub fn check_policy(&self, command: &str) -> PolicyResult {
        for rule in &self.cfg.execution.commands {
            if let Ok(re) = Regex::new(&rule.pattern) {
                if re.is_match(command) {
                    match rule.action.as_str() {
                        "deny" => return PolicyResult::Denied(rule.pattern.clone()),
                        "confirm" => return PolicyResult::Confirm,
                        _ => continue,
                    }
                }
            }
        }
        match self.cfg.execution.default_action.as_str() {
            "deny" => PolicyResult::Denied("default deny".into()),
            "confirm" => PolicyResult::Confirm,
            _ => PolicyResult::Allowed,
        }
    }

    pub async fn execute(
        &self,
        program: &str,
        args: &[String],
        timeout_ms: Option<u64>,
        env: Option<Vec<(String, String)>>,
    ) -> ExecutionOutput {
        let exec_id = Uuid::new_v4();
        let timeout = timeout_ms
            .unwrap_or(self.cfg.execution.default_timeout_ms)
            .min(self.cfg.execution.max_timeout_ms);

        info!("Exec [{exec_id}]: {program} {args:?} timeout={timeout}ms");

        let mut cmd = Command::new(program);
        cmd.args(args);

        if let Some(env_vars) = env {
            for (k, v) in env_vars {
                cmd.env(k, v);
            }
        }

        let result = tokio::time::timeout(Duration::from_millis(timeout), cmd.output()).await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let success = output.status.success();
                let merged = if stderr.is_empty() { stdout.clone() } else { format!("{stdout}\n{stderr}") };

                debug!("Exec [{exec_id}] exit={} out={} err={}", output.status, stdout.len(), stderr.len());

                ExecutionOutput {
                    execution_id: exec_id.to_string(),
                    success,
                    exit_code: output.status.code().unwrap_or(-1),
                    output: merged,
                }
            }
            Ok(Err(e)) => {
                warn!("Exec [{exec_id}] failed: {e}");
                ExecutionOutput {
                    execution_id: exec_id.to_string(),
                    success: false,
                    exit_code: -1,
                    output: e.to_string(),
                }
            }
            Err(_) => {
                warn!("Exec [{exec_id}] timed out after {timeout}ms");
                ExecutionOutput {
                    execution_id: exec_id.to_string(),
                    success: false,
                    exit_code: -1,
                    output: format!("Timed out after {timeout}ms"),
                }
            }
        }
    }

    pub fn apply_path_mappings(&self, path: &str) -> String {
        let mut result = path.to_string();
        for mapping in &self.cfg.path_mappings {
            if result.starts_with(&mapping.from_prefix) {
                result = result.replacen(&mapping.from_prefix, &mapping.to_prefix, 1);
                break;
            }
            if result.starts_with(&mapping.to_prefix) {
                result = result.replacen(&mapping.to_prefix, &mapping.from_prefix, 1);
                break;
            }
        }
        result
    }
}

#[derive(Debug, Clone)]
pub enum PolicyResult {
    Allowed,
    Denied(String),
    Confirm,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExecutionOutput {
    pub execution_id: String,
    pub success: bool,
    pub exit_code: i32,
    pub output: String,
}
