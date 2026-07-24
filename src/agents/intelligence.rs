use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use crate::services::wsl::WslService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub category: String,
    pub priority: String,
    pub title: String,
    pub description: String,
    pub action: String,
    pub expected_impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WslRecommendations {
    pub timestamp: String,
    pub distribution_name: String,
    pub score: i32,
    pub items: Vec<Recommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    pub timestamp: String,
    pub distribution_name: String,
    pub overall_score: i32,
    pub insights: Vec<String>,
    pub bottlenecks: Vec<String>,
    pub prediction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePrediction {
    pub workload: String,
    pub distribution_name: String,
    pub recommended_memory_gb: i32,
    pub recommended_cpu_cores: i32,
    pub recommended_disk_gb: i64,
    pub confidence: f64,
    pub reasoning: String,
}

pub struct IntelligenceService {
    #[allow(dead_code)]
    wsl: Arc<WslService>,
}

impl IntelligenceService {
    pub fn new(wsl: Arc<WslService>) -> Self {
        Self { wsl }
    }

    pub async fn get_recommendations(&self, distro: Option<&str>) -> WslRecommendations {
        info!("Generating recommendations for distro={distro:?}");
        let distro_name = distro.unwrap_or("All").to_string();
        let mut items = Vec::new();

        // Simulated metrics collection
        items.push(Recommendation {
            category: "Memory".into(),
            priority: "High".into(),
            title: "High Memory Usage Detected".into(),
            description: "Current memory usage is 85%.".into(),
            action: "Add 'memory=8GB' to .wslconfig".into(),
            expected_impact: "Improved performance and reduced swapping".into(),
        });

        items.push(Recommendation {
            category: "Network".into(),
            priority: "Low".into(),
            title: "Enable DNS Tunneling".into(),
            description: "Improve network performance.".into(),
            action: "Add 'dnsTunneling=true' to .wslconfig".into(),
            expected_impact: "Faster DNS resolution".into(),
        });

        WslRecommendations {
            timestamp: chrono::Utc::now().to_rfc3339(),
            distribution_name: distro_name,
            score: 75,
            items,
        }
    }

    pub async fn analyze_performance(&self, distro: Option<&str>) -> PerformanceAnalysis {
        info!("Analyzing performance for distro={distro:?}");
        PerformanceAnalysis {
            timestamp: chrono::Utc::now().to_rfc3339(),
            distribution_name: distro.unwrap_or("All").to_string(),
            overall_score: 75,
            insights: vec![
                "CPU usage is within normal range.".into(),
                "Memory pressure detected.".into(),
                "Disk space is adequate.".into(),
            ],
            bottlenecks: vec!["Memory".into()],
            prediction: "Health score: 75/100".into(),
        }
    }

    pub async fn predict_resources(&self, distro: &str, workload: &str) -> ResourcePrediction {
        info!("Predicting resources for {distro} workload={workload}");
        let wl = workload.to_lowercase();

        let (mem, cpu, disk, confidence, reasoning) = if wl.contains("docker") || wl.contains("container") {
            (16, 6, 100, 0.90, "Container workloads require substantial resources")
        } else if wl.contains("compile") || wl.contains("build") {
            (8, 4, 50, 0.85, "Build workloads are CPU and memory intensive")
        } else if wl.contains("data") || wl.contains("ml") || wl.contains("ai") {
            (32, 8, 200, 0.88, "Data science and ML workloads are resource intensive")
        } else if wl.contains("web") || wl.contains("server") {
            (4, 2, 20, 0.80, "Web servers need moderate resources")
        } else {
            (4, 2, 30, 0.60, "General purpose workload estimation")
        };

        ResourcePrediction {
            workload: workload.to_string(),
            distribution_name: distro.to_string(),
            recommended_memory_gb: mem,
            recommended_cpu_cores: cpu,
            recommended_disk_gb: disk,
            confidence,
            reasoning: reasoning.to_string(),
        }
    }
}
