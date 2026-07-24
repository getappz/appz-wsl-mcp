use serde::Serialize;
use tokio::process::Command;

#[derive(Debug, Serialize)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub permissions: String,
    pub modified: String,
    pub file_type: String,
}

#[derive(Debug, Serialize)]
pub struct DirEntry {
    pub name: String,
    pub entry_type: String,
    pub size: Option<u64>,
    pub modified: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TreeEntry {
    pub name: String,
    pub entry_type: String,
    pub children: Option<Vec<TreeEntry>>,
}

#[derive(Debug, Serialize)]
pub struct SearchMatch {
    pub file: String,
    pub line: u32,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct EditResult {
    pub success: bool,
    pub diff: Option<String>,
    pub message: String,
}

pub struct FileOpsService;

impl FileOpsService {
    async fn wsl_exec(args: &[&str]) -> String {
        Command::new("wsl.exe")
            .args(args)
            .output()
            .await
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default()
    }

    async fn wsl_exec_str(args: &[String]) -> String {
        Command::new("wsl.exe")
            .args(args)
            .output()
            .await
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default()
    }

    async fn wsl_exec_with_stderr(args: &[&str]) -> (String, String, bool) {
        match Command::new("wsl.exe").args(args).output().await {
            Ok(o) => (
                String::from_utf8_lossy(&o.stdout).to_string(),
                String::from_utf8_lossy(&o.stderr).to_string(),
                o.status.success(),
            ),
            Err(e) => (String::new(), e.to_string(), false),
        }
    }

    pub async fn read_file(path: &str) -> String {
        Self::wsl_exec(&["cat", path]).await
    }

    pub async fn write_file(path: &str, content: &str) -> (String, bool) {
        let cmd = format!("mkdir -p \"$(dirname '{}')\" && cat > '{}' << 'WSLMCPEOF'\n{}\nWSLMCPEOF", path, path, content);
        let (_stdout, stderr, ok) = Self::wsl_exec_with_stderr(&["sh", "-c", &cmd]).await;
        let msg = if ok {
            format!("Written {} bytes to {}", content.len(), path)
        } else {
            format!("Write failed: {}", stderr)
        };
        (msg, ok)
    }

    pub async fn edit_file(path: &str, edits: &[EditOp], dry_run: bool) -> EditResult {
        let content = Self::read_file(path).await;
        if content.is_empty() {
            return EditResult {
                success: false,
                diff: None,
                message: "File empty or not found".into(),
            };
        }
        let mut result = content.clone();
        let mut diffs = Vec::new();

        for edit in edits {
            if result.contains(&edit.old_text) {
                result = result.replace(&edit.old_text, &edit.new_text);
                if !dry_run {
                    diffs.push(format!("--- replace \"{:.40}\" → \"{:.40}\"", edit.old_text, edit.new_text));
                }
            } else {
                diffs.push(format!("--- pattern not found: \"{:.40}\"", edit.old_text));
            }
        }

        if dry_run {
            let diff = diffs.join("\n");
            return EditResult {
                success: !content.eq(&result),
                diff: Some(if diff.is_empty() { "No changes".into() } else { diff }),
                message: "Dry run".into(),
            };
        }

        Self::write_file(path, &result).await;
        EditResult {
            success: true,
            diff: Some(diffs.join("\n")),
            message: "File updated".into(),
        }
    }

    pub async fn search_files(path: &str, pattern: &str, exclude: &[String]) -> Vec<String> {
        let mut cmd = vec!["find".to_string(), path.to_string(), "-name".to_string(), pattern.to_string(), "-type".to_string(), "f".to_string()];
        for exc in exclude {
            cmd.push("!".to_string());
            cmd.push("-path".to_string());
            cmd.push(format!("*/{}/*", exc));
        }
        let out = Self::wsl_exec_str(&cmd).await;
        out.lines().map(|l| l.to_string()).collect()
    }

    pub async fn search_in_files(
        path: &str,
        pattern: &str,
        is_regex: bool,
        case_insensitive: bool,
        include: &[String],
        exclude: &[String],
        max_results: usize,
        context_lines: usize,
    ) -> Vec<SearchMatch> {
        let mut cmd = vec!["grep".to_string(), "-r".to_string(), "-n".to_string()];
        if is_regex {
            cmd.push("-E".to_string());
        }
        if case_insensitive {
            cmd.push("-i".to_string());
        }
        if context_lines > 0 {
            cmd.push("-C".to_string());
            cmd.push(context_lines.to_string());
        }
        cmd.push("--max-count".to_string());
        cmd.push(max_results.to_string());
        for inc in include {
            cmd.push("--include".to_string());
            cmd.push(inc.clone());
        }
        for exc in exclude {
            cmd.push("--exclude-dir".to_string());
            cmd.push(exc.clone());
        }
        cmd.push(pattern.to_string());
        cmd.push(path.to_string());

        let out = Self::wsl_exec_str(&cmd).await;
        out.lines().filter_map(|line| {
            let mut parts = line.splitn(3, ':');
            let file = parts.next()?.to_string();
            let line_num: u32 = parts.next()?.parse().ok()?;
            let content = parts.next()?.to_string();
            Some(SearchMatch { file, line: line_num, content })
        }).collect()
    }

    pub async fn list_directory(path: &str) -> Vec<DirEntry> {
        let out = Self::wsl_exec(&["ls", "-la", path]).await;
        out.lines().skip(1).filter_map(|line| {
            if line.is_empty() { return None; }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 9 { return None; }
            let name = parts.last().copied().unwrap_or("");
            if name == "." || name == ".." { return None; }
            let entry_type = if parts[0].starts_with('d') { "DIR" } else { "FILE" };
            let size: Option<u64> = parts[4].parse().ok();
            let modified = Some(format!("{} {} {}", parts[5], parts[6], parts[7]));
            Some(DirEntry {
                name: name.to_string(),
                entry_type: entry_type.to_string(),
                size,
                modified,
            })
        }).collect()
    }

    pub async fn directory_tree(path: &str) -> Vec<TreeEntry> {
        let out = Self::wsl_exec(&["find", path, "-maxdepth", "3", "-printf", "%y:%p\\n"]).await;
        let mut entries: Vec<TreeEntry> = Vec::new();
        let path_base = path.trim_end_matches('/');
        for line in out.lines() {
            if line.is_empty() { continue; }
            let (ty, full_path) = match line.split_once(':') {
                Some((t, p)) => (t, p),
                None => continue,
            };
            let relative = full_path.strip_prefix(path_base).unwrap_or(full_path).trim_start_matches('/');
            if relative.is_empty() { continue; }
            let entry_type = if ty == "d" { "DIR" } else { "FILE" };
            let parts: Vec<&str> = relative.split('/').collect();
            Self::insert_tree(&mut entries, &parts, entry_type);
        }
        entries
    }

    fn insert_tree(entries: &mut Vec<TreeEntry>, parts: &[&str], entry_type: &str) {
        if parts.is_empty() { return; }
        let name = parts[0].to_string();
        if parts.len() == 1 {
            if !entries.iter().any(|e| e.name == name) {
                entries.push(TreeEntry {
                    name,
                    entry_type: entry_type.to_string(),
                    children: None,
                });
            }
        } else {
            let pos = entries.iter().position(|e| e.name == name);
            if let Some(idx) = pos {
                if entries[idx].children.is_none() {
                    entries[idx].children = Some(Vec::new());
                }
                if let Some(children) = &mut entries[idx].children {
                    Self::insert_tree(children, &parts[1..], entry_type);
                }
            } else {
                let mut children = Vec::new();
                Self::insert_tree(&mut children, &parts[1..], entry_type);
                entries.push(TreeEntry {
                    name,
                    entry_type: "DIR".to_string(),
                    children: Some(children),
                });
            }
        }
    }

    pub async fn get_file_info(path: &str) -> FileInfo {
        let stat = Self::wsl_exec(&["stat", "--format", "%s:%A:%y:%F", path]).await;
        let parts: Vec<&str> = stat.trim().splitn(4, ':').collect();
        FileInfo {
            path: path.to_string(),
            size: parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0),
            permissions: parts.get(1).unwrap_or(&"").to_string(),
            modified: parts.get(2).unwrap_or(&"").to_string(),
            file_type: parts.get(3).unwrap_or(&"").to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EditOp {
    pub old_text: String,
    pub new_text: String,
}
