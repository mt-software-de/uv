use std::path::{Path, PathBuf};

use uv_static::EnvVars;

/// A collection of `.env` file paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvFile {
    paths: Vec<PathBuf>,
    /// Whether auto-discovery should be disabled (e.g., when --no-env-file is set)
    no_auto_discovery: bool,
}

impl Default for EnvFile {
    fn default() -> Self {
        Self {
            paths: Vec::new(),
            no_auto_discovery: false,
        }
    }
}

impl EnvFile {
    /// Parse the env file paths from command-line arguments.
    /// 
    /// If no explicit files are provided, `no_env_file` is not set, and the
    /// `UV_AUTO_LOAD_ENV_FILES` environment variable is set to `true`, a placeholder
    /// is returned that will trigger auto-discovery later when the project directory is known.
    pub fn from_args(env_file: Vec<String>, no_env_file: bool) -> Self {
        if no_env_file {
            return Self {
                paths: Vec::new(),
                no_auto_discovery: true,
            };
        }

        let mut paths = Vec::new();

        // Split on spaces, but respect backslashes.
        for env_file in env_file {
            let mut current = String::new();
            let mut escape = false;
            for c in env_file.chars() {
                if escape {
                    current.push(c);
                    escape = false;
                } else if c == '\\' {
                    escape = true;
                } else if c.is_whitespace() {
                    if !current.is_empty() {
                        paths.push(PathBuf::from(current));
                        current = String::new();
                    }
                } else {
                    current.push(c);
                }
            }
            if !current.is_empty() {
                paths.push(PathBuf::from(current));
            }
        }

        Self {
            paths,
            no_auto_discovery: false,
        }
    }

    /// Auto-discover `.env*` files in the given directory if auto-loading is enabled
    /// and no explicit files were provided.
    /// 
    /// This should be called with the project directory after it's been determined.
    pub fn resolve_auto_discovery(&mut self, project_dir: &Path) {
        // Only auto-discover if:
        // 1. No explicit files were provided
        // 2. Auto-discovery is not disabled (e.g., via --no-env-file)
        // 3. UV_AUTO_LOAD_ENV_FILES is set to true
        if !self.paths.is_empty() || self.no_auto_discovery || !should_auto_load_env_files() {
            return;
        }

        // Search for .env* files in the project directory in order of precedence
        // (first file has lowest precedence)
        // This order ensures that more specific files override general ones
        let env_patterns = [".env", ".env.local", ".env.production"];
        for pattern in &env_patterns {
            let path = project_dir.join(pattern);
            if path.exists() {
                self.paths.push(path);
            }
        }
    }

    /// Iterate over the paths in the env file.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &PathBuf> {
        self.paths.iter()
    }
}

/// Check if auto-loading of `.env*` files is enabled via environment variable.
fn should_auto_load_env_files() -> bool {
    std::env::var(EnvVars::UV_AUTO_LOAD_ENV_FILES)
        .ok()
        .and_then(|value| {
            let value = value.trim().to_lowercase();
            match value.as_str() {
                "true" | "1" | "yes" | "y" | "on" => Some(true),
                "false" | "0" | "no" | "n" | "off" => Some(false),
                _ => None,
            }
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_args_no_auto_discovery_by_default() {
        // When no explicit files provided and UV_AUTO_LOAD_ENV_FILES is not set,
        // no auto-discovery should happen
        let env_file = EnvFile::from_args(vec![], false);
        assert_eq!(env_file, EnvFile::default());
    }

    #[test]
    fn test_from_args_no_env_file() {
        let env_file = EnvFile::from_args(vec!["path1 path2".to_string()], true);
        assert_eq!(
            env_file,
            EnvFile {
                paths: Vec::new(),
                no_auto_discovery: true,
            }
        );
    }

    #[test]
    fn test_from_args_empty_string() {
        // Empty string is treated as no explicit files, no auto-discovery without env var
        let env_file = EnvFile::from_args(vec![String::new()], false);
        assert_eq!(env_file, EnvFile::default());
    }

    #[test]
    fn test_from_args_whitespace_only() {
        // Whitespace-only string is treated as no explicit files, no auto-discovery without env var
        let env_file = EnvFile::from_args(vec!["   ".to_string()], false);
        assert_eq!(env_file, EnvFile::default());
    }

    #[test]
    fn test_from_args_single_path() {
        let env_file = EnvFile::from_args(vec!["path1".to_string()], false);
        assert_eq!(
            env_file,
            EnvFile {
                paths: vec![PathBuf::from("path1")],
                no_auto_discovery: false,
            }
        );
    }

    #[test]
    fn test_from_args_multiple_paths() {
        let env_file = EnvFile::from_args(vec!["path1 path2 path3".to_string()], false);
        assert_eq!(
            env_file,
            EnvFile {
                paths: vec![
                    PathBuf::from("path1"),
                    PathBuf::from("path2"),
                    PathBuf::from("path3")
                ],
                no_auto_discovery: false,
            }
        );
    }

    #[test]
    fn test_from_args_escaped_spaces() {
        let env_file = EnvFile::from_args(vec![r"path\ with\ spaces".to_string()], false);
        assert_eq!(
            env_file,
            EnvFile {
                paths: vec![PathBuf::from("path with spaces")],
                no_auto_discovery: false,
            }
        );
    }

    #[test]
    fn test_from_args_mixed_escaped_and_normal() {
        let env_file =
            EnvFile::from_args(vec![r"path1 path\ with\ spaces path2".to_string()], false);
        assert_eq!(
            env_file,
            EnvFile {
                paths: vec![
                    PathBuf::from("path1"),
                    PathBuf::from("path with spaces"),
                    PathBuf::from("path2")
                ],
                no_auto_discovery: false,
            }
        );
    }

    #[test]
    fn test_from_args_escaped_backslash() {
        let env_file = EnvFile::from_args(vec![r"path\\with\\backslashes".to_string()], false);
        assert_eq!(
            env_file,
            EnvFile {
                paths: vec![PathBuf::from(r"path\with\backslashes")],
                no_auto_discovery: false,
            }
        );
    }

    #[test]
    fn test_iter() {
        let env_file = EnvFile {
            paths: vec![PathBuf::from("path1"), PathBuf::from("path2")],
            no_auto_discovery: false,
        };
        let paths: Vec<_> = env_file.iter().collect();
        assert_eq!(
            paths,
            vec![&PathBuf::from("path1"), &PathBuf::from("path2")]
        );
    }
}
