use serde::Serialize;
use std::collections::HashMap;
use tokio::process::Command;

#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub kernel: String,
    pub hostname: String,
    pub os: String,
}

#[derive(Debug, Serialize)]
pub struct OsRelease {
    pub name: String,
    pub version: String,
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct MountInfo {
    pub filesystem: String,
    pub mount_point: String,
    pub fstype: String,
    pub options: String,
}

#[derive(Debug, Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub user: String,
    pub cpu: f64,
    pub memory: f64,
    pub name: String,
    pub command: String,
}

#[derive(Debug, Serialize)]
pub struct DiskUsage {
    pub filesystem: String,
    pub size: String,
    pub used: String,
    pub avail: String,
    pub use_percent: String,
    pub mount: String,
}

#[derive(Debug, Serialize)]
pub struct ShellEntry {
    pub path: String,
    pub available: bool,
}

pub async fn get_system_info() -> SystemInfo {
    let uname = exec_wsl_read(&["uname", "-a"]).await;
    let hostname = exec_wsl_read(&["hostname"]).await;
    let os = exec_wsl_read(&["cat", "/etc/os-release"]).await;
    SystemInfo {
        kernel: uname.trim().to_string(),
        hostname: hostname.trim().to_string(),
        os: os.trim().to_string(),
    }
}

pub async fn get_os_release() -> OsRelease {
    let content = exec_wsl_read(&["cat", "/etc/os-release"]).await;
    let mut name = String::new();
    let mut version = String::new();
    let mut id = String::new();
    for line in content.lines() {
        if let Some((k, v)) = line.split_once('=') {
            let val = v.trim_matches('"');
            match k {
                "NAME" => name = val.to_string(),
                "VERSION_ID" => version = val.to_string(),
                "ID" => id = val.to_string(),
                _ => {}
            }
        }
    }
    OsRelease { name, version, id }
}

pub async fn get_mounts() -> Vec<MountInfo> {
    let content = exec_wsl_read(&["cat", "/proc/mounts"]).await;
    content
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                Some(MountInfo {
                    filesystem: parts[0].to_string(),
                    mount_point: parts[1].to_string(),
                    fstype: parts[2].to_string(),
                    options: parts[3].to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

pub async fn get_wsl_config() -> String {
    exec_wsl_read(&["cat", "/etc/wsl.conf"]).await.trim().to_string()
}

pub async fn get_disk_usage(path: &str) -> Vec<DiskUsage> {
    let output = exec_wsl_read(&["df", "-h", path]).await;
    output
        .lines()
        .skip(1)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                Some(DiskUsage {
                    filesystem: parts[0].to_string(),
                    size: parts[1].to_string(),
                    used: parts[2].to_string(),
                    avail: parts[3].to_string(),
                    use_percent: parts[4].to_string(),
                    mount: parts[5].to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

pub async fn get_env(filter: Option<&str>) -> HashMap<String, String> {
    let output = exec_wsl_read(&["printenv"]).await;
    let mut env = HashMap::new();
    for line in output.lines() {
        if let Some((k, v)) = line.split_once('=') {
            if filter.map_or(true, |f| k.to_lowercase().contains(&f.to_lowercase())) {
                env.insert(k.to_string(), v.to_string());
            }
        }
    }
    env
}

pub async fn list_processes(filter: Option<&str>) -> Vec<ProcessInfo> {
    let output = exec_wsl_read(&["ps", "aux"]).await;
    output
        .lines()
        .skip(1)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 11 {
                let cmd = parts[10..].join(" ");
                let name = parts[10].to_string();
                let matched = filter.map_or(true, |f| {
                    name.to_lowercase().contains(&f.to_lowercase())
                        || cmd.to_lowercase().contains(&f.to_lowercase())
                });
                if matched {
                    Some(ProcessInfo {
                        pid: parts[1].parse().unwrap_or(0),
                        user: parts[0].to_string(),
                        cpu: parts[2].parse().unwrap_or(0.0),
                        memory: parts[3].parse().unwrap_or(0.0),
                        name,
                        command: cmd,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

pub async fn get_package_managers() -> Vec<String> {
    let candidates = ["pacman", "apt", "dnf", "apk", "cargo", "npm", "pip", "uv", "brew", "snap"];
    let mut found = Vec::new();
    for pm in &candidates {
        let output = exec_wsl_read(&["which", pm]).await;
        if !output.trim().is_empty() {
            found.push(pm.to_string());
        }
    }
    found
}

pub async fn get_shells() -> Vec<ShellEntry> {
    let content = exec_wsl_read(&["cat", "/etc/shells"]).await;
    content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
        .map(|l| ShellEntry {
            path: l.trim().to_string(),
            available: true,
        })
        .collect()
}

pub async fn get_default_shell() -> String {
    exec_wsl_read(&["printenv", "SHELL"]).await.trim().to_string()
}

async fn exec_wsl_read(args: &[&str]) -> String {
    Command::new("wsl.exe")
        .args(args)
        .output()
        .await
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default()
}
