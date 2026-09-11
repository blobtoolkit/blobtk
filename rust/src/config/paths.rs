use std::path::{Component, Path, PathBuf};

use crate::config::schema::ResolvedPathConfig;

fn is_remote_url(path: &PathBuf) -> bool {
    let value = path.to_string_lossy();
    value.starts_with("http://")
        || value.starts_with("https://")
        || value.starts_with("ftp://")
        || value.starts_with("ssh://")
}

fn expand_home_path(path: &Path) -> PathBuf {
    if let Some(first) = path.components().next() {
        if matches!(first, Component::Normal(part) if part == "~") {
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(home).join(path.strip_prefix("~").unwrap_or(path));
            }
        }
    }
    path.to_path_buf()
}

pub fn resolve_source_path(source: &ResolvedPathConfig) -> Result<PathBuf, anyhow::Error> {
    let local_path = source.local_path.as_ref().map(|p| expand_home_path(p));
    let path = source.path.as_ref().map(|p| expand_home_path(p));

    if let Some(local_path) = &local_path {
        let value = local_path.to_string_lossy();
        if value.contains('{') || value.contains('}') {
            return Ok(local_path.clone());
        }
        if local_path.exists() {
            return Ok(local_path.clone());
        }
        if is_remote_url(local_path) {
            return Ok(local_path.clone());
        }
    }

    if let Some(path) = &path {
        let value = path.to_string_lossy();
        if value.contains('{') || value.contains('}') {
            return Ok(path.clone());
        }
        if path.exists() {
            return Ok(path.clone());
        }
        if is_remote_url(path) {
            return Ok(path.clone());
        }
    }

    let preferred = local_path
        .clone()
        .or_else(|| path.clone())
        .unwrap_or_else(|| PathBuf::from(""));

    Err(anyhow::anyhow!(
        "source path is missing or unreadable: {}",
        preferred.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_local_path_when_present_and_readable() {
        let tmp_dir = std::env::temp_dir().join("blobtk-config-paths");
        let _ = std::fs::create_dir_all(&tmp_dir);
        let local_path = tmp_dir.join("local.txt");
        let remote_path = tmp_dir.join("remote.txt");
        std::fs::write(&local_path, "local").unwrap();
        std::fs::write(&remote_path, "remote").unwrap();

        let source = ResolvedPathConfig {
            path: Some(remote_path.clone()),
            local_path: Some(local_path.clone()),
        };

        let resolved = resolve_source_path(&source).unwrap();
        assert_eq!(resolved, local_path);
    }

    #[test]
    fn expands_home_relative_local_paths() {
        let home = std::env::var_os("HOME").expect("HOME should be set");
        let expected = PathBuf::from(home).join("tmp").join("example.txt");
        std::fs::create_dir_all(expected.parent().unwrap()).unwrap();
        std::fs::write(&expected, "hello").unwrap();

        let source = ResolvedPathConfig {
            path: None,
            local_path: Some(PathBuf::from("~/tmp/example.txt")),
        };

        let resolved = resolve_source_path(&source).unwrap();
        assert_eq!(resolved, expected);
    }

    #[test]
    fn accepts_remote_url_paths() {
        let source = ResolvedPathConfig {
            path: Some(PathBuf::from("https://example.org/data/track.bed.gz")),
            local_path: None,
        };

        let resolved = resolve_source_path(&source).unwrap();
        assert_eq!(
            resolved,
            PathBuf::from("https://example.org/data/track.bed.gz")
        );
    }

    #[test]
    fn accepts_ssh_url_paths() {
        let source = ResolvedPathConfig {
            path: Some(PathBuf::from("ssh://example.org/data/track.bed.gz")),
            local_path: None,
        };

        let resolved = resolve_source_path(&source).unwrap();
        assert_eq!(
            resolved,
            PathBuf::from("ssh://example.org/data/track.bed.gz")
        );
    }
}
