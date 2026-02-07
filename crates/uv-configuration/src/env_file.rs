use std::path::PathBuf;

/// A collection of `.env` file paths.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct EnvFile(Vec<PathBuf>);

impl EnvFile {
    /// Parse the env file paths from command-line arguments.
    /// 
    /// If no explicit files are provided and `no_env_file` is not set,
    /// automatically discovers and uses `.env*` files in the current directory.
    pub fn from_args(env_file: Vec<String>, no_env_file: bool) -> Self {
        if no_env_file {
            return Self::default();
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

        // Auto-discover .env* files if no explicit files provided
        if paths.is_empty() {
            // Search for .env* files in order of precedence (first file has lowest precedence)
            // This order ensures that more specific files override general ones
            let env_patterns = [".env", ".env.local", ".env.production"];
            for pattern in &env_patterns {
                let path = PathBuf::from(pattern);
                if path.exists() {
                    paths.push(path);
                }
            }
        }

        Self(paths)
    }

    /// Iterate over the paths in the env file.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &PathBuf> {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_args_auto_discovery() {
        // When no explicit files provided, auto-discovery happens
        // The actual files found will depend on what exists in the current directory
        // This test just verifies the function doesn't panic
        let env_file = EnvFile::from_args(vec![], false);
        // The result may or may not be empty depending on whether .env* files exist
        // We just verify it doesn't error
        let _ = env_file.iter().count();
    }

    #[test]
    fn test_from_args_no_env_file() {
        let env_file = EnvFile::from_args(vec!["path1 path2".to_string()], true);
        assert_eq!(env_file, EnvFile::default());
    }

    #[test]
    fn test_from_args_empty_string() {
        // Empty string is treated as no explicit files, so auto-discovery happens
        let env_file = EnvFile::from_args(vec![String::new()], false);
        // The result may or may not be empty depending on whether .env* files exist
        let _ = env_file.iter().count();
    }

    #[test]
    fn test_from_args_whitespace_only() {
        // Whitespace-only string is treated as no explicit files, so auto-discovery happens
        let env_file = EnvFile::from_args(vec!["   ".to_string()], false);
        // The result may or may not be empty depending on whether .env* files exist
        let _ = env_file.iter().count();
    }

    #[test]
    fn test_from_args_single_path() {
        let env_file = EnvFile::from_args(vec!["path1".to_string()], false);
        assert_eq!(env_file.0, vec![PathBuf::from("path1")]);
    }

    #[test]
    fn test_from_args_multiple_paths() {
        let env_file = EnvFile::from_args(vec!["path1 path2 path3".to_string()], false);
        assert_eq!(
            env_file.0,
            vec![
                PathBuf::from("path1"),
                PathBuf::from("path2"),
                PathBuf::from("path3")
            ]
        );
    }

    #[test]
    fn test_from_args_escaped_spaces() {
        let env_file = EnvFile::from_args(vec![r"path\ with\ spaces".to_string()], false);
        assert_eq!(env_file.0, vec![PathBuf::from("path with spaces")]);
    }

    #[test]
    fn test_from_args_mixed_escaped_and_normal() {
        let env_file =
            EnvFile::from_args(vec![r"path1 path\ with\ spaces path2".to_string()], false);
        assert_eq!(
            env_file.0,
            vec![
                PathBuf::from("path1"),
                PathBuf::from("path with spaces"),
                PathBuf::from("path2")
            ]
        );
    }

    #[test]
    fn test_from_args_escaped_backslash() {
        let env_file = EnvFile::from_args(vec![r"path\\with\\backslashes".to_string()], false);
        assert_eq!(env_file.0, vec![PathBuf::from(r"path\with\backslashes")]);
    }

    #[test]
    fn test_iter() {
        let env_file = EnvFile(vec![PathBuf::from("path1"), PathBuf::from("path2")]);
        let paths: Vec<_> = env_file.iter().collect();
        assert_eq!(
            paths,
            vec![&PathBuf::from("path1"), &PathBuf::from("path2")]
        );
    }
}
