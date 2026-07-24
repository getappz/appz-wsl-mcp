use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use crate::services::wsl::WslService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTaskResult {
    pub success: bool,
    pub task: String,
    pub steps: Vec<String>,
    pub results: Vec<String>,
    pub summary: String,
}

pub struct TaskPlanner {
    #[allow(dead_code)]
    wsl: Arc<WslService>,
}

impl TaskPlanner {
    pub fn new(wsl: Arc<WslService>) -> Self {
        Self { wsl }
    }

    pub async fn execute_task(&self, task: &str, context: Option<&str>) -> AgentTaskResult {
        info!("Agent planning: {task}");
        let steps = self.plan(task, context);
        let mut results = Vec::new();

        for step in &steps {
            info!("Executing step: {step}");
            results.push(self.run_step(step).await);
        }

        let n = steps.len();
        AgentTaskResult {
            success: results.iter().all(|r| !r.starts_with("Failed")),
            task: task.to_string(),
            steps,
            results,
            summary: format!("Completed {n} steps"),
        }
    }

    fn plan(&self, task: &str, _context: Option<&str>) -> Vec<String> {
        let t = task.to_lowercase();
        if t.contains("install") && t.contains("distro") {
            vec![
                "Check if distribution is already installed".into(),
                "Install the requested distribution".into(),
                "Verify installation success".into(),
                "Start the distribution".into(),
            ]
        } else if t.contains("setup") || t.contains("configure") {
            vec![
                "Get current WSL configuration".into(),
                "Apply requested configuration changes".into(),
                "Restart WSL to apply changes".into(),
                "Verify configuration".into(),
            ]
        } else if t.contains("backup") || t.contains("export") {
            vec![
                "List available distributions".into(),
                "Stop the distribution".into(),
                "Export to tar file".into(),
                "Verify export completed".into(),
            ]
        } else if t.contains("optimize") || t.contains("performance") {
            vec![
                "Analyze current resource usage".into(),
                "Generate optimization recommendations".into(),
                "Apply performance tuning".into(),
                "Validate improvements".into(),
            ]
        } else {
            vec![
                format!("Analyze requirement: {task}"),
                "Execute appropriate WSL commands".into(),
                "Verify results".into(),
            ]
        }
    }

    async fn run_step(&self, step: &str) -> String {
        // Placeholder — in production this would call wsl service methods
        format!("Completed: {step}")
    }
}
