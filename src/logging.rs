use crate::services::exec::ExecutionOutput;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
pub struct ExecutionLogger {
    dir: PathBuf,
    persist: bool,
}

impl ExecutionLogger {
    pub fn new(dir: &str, persist: bool) -> Self {
        Self {
            dir: PathBuf::from(dir),
            persist,
        }
    }

    pub async fn log(&self, output: &ExecutionOutput) {
        if !self.persist {
            return;
        }
        let _ = fs::create_dir_all(&self.dir).await;
        let path = self.dir.join(format!("{}.json", output.execution_id));
        if let Ok(json) = serde_json::to_string_pretty(output) {
            let _ = fs::write(&path, &json).await;
            info!("Execution log written to {path:?}");
        }
    }

    pub async fn list_logs(&self) -> Vec<PathBuf> {
        let mut entries = match fs::read_dir(&self.dir).await {
            Ok(d) => d,
            Err(_) => return Vec::new(),
        };
        let mut logs = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            if entry.path().extension().map_or(false, |e| e == "json") {
                logs.push(entry.path());
            }
        }
        logs.sort();
        logs
    }
}
