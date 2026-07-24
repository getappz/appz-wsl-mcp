use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub execution: ExecutionConfig,
    pub auth: AuthConfig,
    pub logging: LogConfig,
    pub path_mappings: Vec<PathMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub transport: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub default_action: String,
    pub default_timeout_ms: u64,
    pub max_timeout_ms: u64,
    pub commands: Vec<CommandPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPolicy {
    pub pattern: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub api_key_env: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub dir: String,
    pub persist: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathMapping {
    pub from_prefix: String,
    pub to_prefix: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                address: "127.0.0.1".into(),
                port: 8787,
                transport: "stdio".into(),
            },
            execution: ExecutionConfig {
                default_action: "allow".into(),
                default_timeout_ms: 30000,
                max_timeout_ms: 300000,
                commands: vec![
                    CommandPolicy {
                        pattern: "shutdown|reboot|poweroff|halt".into(),
                        action: "deny".into(),
                    },
                ],
            },
            auth: AuthConfig { api_key_env: None },
            logging: LogConfig {
                dir: ".wsl-mcp-logs".into(),
                persist: false,
            },
            path_mappings: vec![
                PathMapping {
                    from_prefix: "C:\\Users\\".into(),
                    to_prefix: "/mnt/c/Users/".into(),
                },
            ],
        }
    }
}

pub async fn load(path: &str) -> AppConfig {
    if Path::new(path).exists() {
        let contents = fs::read_to_string(path).unwrap_or_default();
        serde_yaml::from_str(&contents).unwrap_or_default()
    } else {
        let cfg = AppConfig::default();
        let yaml = serde_yaml::to_string(&cfg).unwrap();
        let _ = fs::write(path, &yaml);
        cfg
    }
}
