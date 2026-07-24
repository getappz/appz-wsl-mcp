use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WslConfig {
    pub memory: Option<String>,
    pub processors: Option<String>,
    pub swap: Option<String>,
    pub localhost_forwarding: Option<bool>,
    pub nested_virtualization: Option<bool>,
    pub vm_idle_timeout: Option<u64>,
    pub dns_tunneling: Option<bool>,
}

pub struct WslConfigService;

impl WslConfigService {
    pub fn wslconfig_path() -> String {
        let profile = std::env::var("USERPROFILE").unwrap_or_else(|_| r"C:\Users\Default".into());
        format!(r"{}\.wslconfig", profile)
    }

    pub async fn read() -> WslConfig {
        let path = Self::wslconfig_path();
        let path_ref = Path::new(&path);

        if !path_ref.exists() {
            info!("No .wslconfig found at {path}");
            return WslConfig {
                memory: None,
                processors: None,
                swap: None,
                localhost_forwarding: None,
                nested_virtualization: None,
                vm_idle_timeout: None,
                dns_tunneling: None,
            };
        }

        match fs::read_to_string(path_ref).await {
            Ok(content) => Self::parse(&content),
            Err(e) => {
                warn!("Failed to read .wslconfig: {e}");
                WslConfig {
                    memory: None,
                    processors: None,
                    swap: None,
                    localhost_forwarding: None,
                    nested_virtualization: None,
                    vm_idle_timeout: None,
                    dns_tunneling: None,
                }
            }
        }
    }

    fn parse(content: &str) -> WslConfig {
        let mut cfg = WslConfig {
            memory: None,
            processors: None,
            swap: None,
            localhost_forwarding: None,
            nested_virtualization: None,
            vm_idle_timeout: None,
            dns_tunneling: None,
        };

        let mut in_wsl2 = false;
        for line in content.lines() {
            let line = line.trim();
            if line.eq_ignore_ascii_case("[wsl2]") {
                in_wsl2 = true;
                continue;
            }
            if line.starts_with('[') {
                in_wsl2 = false;
                continue;
            }
            if !in_wsl2 {
                continue;
            }
            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim().to_lowercase();
                let val = val.trim();
                match key.as_str() {
                    "memory" => cfg.memory = Some(val.to_string()),
                    "processors" => cfg.processors = Some(val.to_string()),
                    "swap" => cfg.swap = Some(val.to_string()),
                    "localhostforwarding" => cfg.localhost_forwarding = Some(val.eq_ignore_ascii_case("true")),
                    "nestedvirtualization" => cfg.nested_virtualization = Some(val.eq_ignore_ascii_case("true")),
                    "vmidletimeout" => cfg.vm_idle_timeout = val.parse::<u64>().ok(),
                    "dnstunneling" => cfg.dns_tunneling = Some(val.eq_ignore_ascii_case("true")),
                    _ => {}
                }
            }
        }
        cfg
    }

    pub async fn write(settings: &HashMap<String, String>) -> Result<String, String> {
        let path = Self::wslconfig_path();
        let mut content = String::from("[wsl2]\n");
        for (k, v) in settings {
            content.push_str(&format!("{k}={v}\n"));
        }
        fs::write(&path, &content)
            .await
            .map_err(|e| e.to_string())?;
        info!("Written .wslconfig to {path}");
        Ok(path)
    }
}
