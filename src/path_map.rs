use crate::config::PathMapping;

pub fn apply(path: &str, mappings: &[PathMapping]) -> String {
    let mut result = path.to_string();
    for mapping in mappings {
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

pub fn windows_to_wsl(path: &str) -> String {
    if path.len() > 2 && path.as_bytes()[1] == b':' {
        let drive = path[..1].to_lowercase();
        let rest = path[2..].replace('\\', "/");
        format!("/mnt/{drive}{rest}")
    } else {
        path.to_string()
    }
}

pub fn wsl_to_windows(path: &str) -> String {
    if path.starts_with("/mnt/") && path.len() > 5 {
        let drive = &path[5..6];
        let rest = &path[6..].replace('/', "\\");
        format!("{drive}:{rest}")
    } else {
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_to_wsl() {
        assert_eq!(windows_to_wsl(r"C:\Users\test"), "/mnt/c/Users/test");
        assert_eq!(windows_to_wsl(r"D:\projects"), "/mnt/d/projects");
    }

    #[test]
    fn test_wsl_to_windows() {
        assert_eq!(wsl_to_windows("/mnt/c/Users/test"), r"C:\Users\test");
        assert_eq!(wsl_to_windows("/mnt/d/projects"), r"D:\projects");
    }
}
