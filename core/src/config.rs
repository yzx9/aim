// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;
use std::path::{Path, PathBuf};

use crate::{DateTimeAnchor, Priority};

/// The name of the AIM application.
pub const APP_NAME: &str = "aim";

/// Configuration for the AIM application.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    /// Path to the calendar directory.
    pub calendar_path: PathBuf,

    /// Directory for storing application state.
    #[serde(default)]
    pub state_dir: Option<PathBuf>,

    /// Default due time for new tasks.
    #[serde(default)]
    pub default_due: Option<DateTimeAnchor>,

    /// Default priority for new tasks.
    #[serde(default)]
    pub default_priority: Priority,

    /// If true, items with no priority will be listed first.
    #[serde(default)]
    pub default_priority_none_fist: bool,
}

impl Config {
    /// Normalize the configuration.
    ///
    /// # Errors
    /// If path normalization fails.
    #[tracing::instrument(skip(self))]
    pub fn normalize(&mut self) -> Result<(), Box<dyn Error>> {
        // Normalize calendar path
        self.calendar_path = expand_path(&self.calendar_path)?;

        // Normalize state directory
        match &self.state_dir {
            Some(a) => {
                let state_dir = expand_path(a)
                    .map_err(|e| format!("Failed to expand state directory path: {e}"))?;
                self.state_dir = Some(state_dir);
            }
            None => match get_state_dir() {
                Ok(a) => self.state_dir = Some(a.join(APP_NAME)),
                Err(err) => tracing::warn!(err, "failed to get state directory"),
            },
        }

        Ok(())
    }
}

/// Handle tilde (~) and environment variables in the path
fn expand_path(path: &Path) -> Result<PathBuf, Box<dyn Error>> {
    if path.is_absolute() {
        return Ok(path.to_owned());
    }

    let path = path.to_str().ok_or("Invalid path")?;

    // Handle tilde and home directory
    let home_prefixes: &[&str] = if cfg!(unix) {
        &["~/", "$HOME/", "${HOME}/"]
    } else {
        &[r"~\", "~/", r"%UserProfile%\", r"%UserProfile%/"]
    };
    for prefix in home_prefixes {
        if let Some(stripped) = path.strip_prefix(prefix) {
            return Ok(get_home_dir()?.join(stripped));
        }
    }

    // Handle config directories
    let config_prefixes: &[&str] = if cfg!(unix) {
        &["$XDG_CONFIG_HOME/", "${XDG_CONFIG_HOME}/"]
    } else {
        &[r"%LOCALAPPDATA%\", "%LOCALAPPDATA%/"]
    };
    for prefix in config_prefixes {
        if let Some(stripped) = path.strip_prefix(prefix) {
            return Ok(get_config_dir()?.join(stripped));
        }
    }

    Ok(path.into())
}

fn get_home_dir() -> Result<PathBuf, Box<dyn Error>> {
    dirs::home_dir().ok_or_else(|| "User-specific home directory not found".into())
}

fn get_config_dir() -> Result<PathBuf, Box<dyn Error>> {
    #[cfg(unix)]
    let config_dir = xdg::BaseDirectories::new().get_config_home();
    #[cfg(windows)]
    let config_dir = dirs::config_dir();
    config_dir.ok_or_else(|| "User-specific home directory not found".into())
}

fn get_state_dir() -> Result<PathBuf, Box<dyn Error>> {
    #[cfg(unix)]
    let state_dir = xdg::BaseDirectories::new().get_state_home();
    #[cfg(windows)]
    let state_dir = dirs::data_dir();
    state_dir.ok_or_else(|| "User-specific state directory not found".into())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn parses_full_toml_config() {
        const TOML: &str = r#"
calendar_path = "calendar"
state_dir = "state"
default_due = "1d"
default_priority = "high"
default_priority_none_fist = true
"#;

        let config: Config = toml::from_str(TOML).expect("Failed to parse TOML");
        assert_eq!(config.calendar_path, PathBuf::from("calendar"));
        assert_eq!(config.state_dir, Some(PathBuf::from("state")));
        assert_eq!(config.default_due, Some(DateTimeAnchor::InDays(1)));
        assert_eq!(config.default_priority, Priority::P2);
        assert!(config.default_priority_none_fist);
    }

    #[test]
    fn parses_minimal_toml_with_defaults() {
        const TOML: &str = r#"
calendar_path = "calendar"
"#;

        let config: Config = toml::from_str(TOML).expect("Failed to parse TOML");
        assert_eq!(config.calendar_path, PathBuf::from("calendar"));
        assert_eq!(config.state_dir, None);
        assert_eq!(config.default_due, None);
        assert_eq!(config.default_priority, Priority::None);
        assert!(!config.default_priority_none_fist);
    }

    #[test]
    fn expands_path_with_home_env_vars() {
        let home = get_home_dir().unwrap();
        let home_prefixes: &[&str] = if cfg!(unix) {
            &["~", "$HOME", "${HOME}"]
        } else {
            &[r"~", r"%UserProfile%"]
        };
        for prefix in home_prefixes {
            let result = expand_path(&PathBuf::from(format!("{prefix}/Documents"))).unwrap();
            assert_eq!(result, home.join("Documents"));
            assert!(result.is_absolute());
        }
    }

    #[test]
    fn expands_path_with_config_env_vars() {
        let config_dir = get_config_dir().unwrap();
        let config_prefixes: &[&str] = if cfg!(unix) {
            &["$XDG_CONFIG_HOME", "${XDG_CONFIG_HOME}"]
        } else {
            &[r"%LOCALAPPDATA%"]
        };
        for prefix in config_prefixes {
            let result = expand_path(&PathBuf::from(format!("{prefix}/config.toml"))).unwrap();
            assert_eq!(result, config_dir.join("config.toml"));
            assert!(result.is_absolute());
        }
    }

    #[test]
    fn preserves_absolute_path() {
        let absolute_path = PathBuf::from("/etc/passwd");
        let result = expand_path(&absolute_path).unwrap();
        assert_eq!(result, absolute_path);
    }

    #[test]
    fn preserves_relative_path() {
        let relative_path = PathBuf::from("relative/path/to/file");
        let result = expand_path(&relative_path).unwrap();
        assert_eq!(result, relative_path);
    }

    #[test]
    fn parses_datetime_anchor_with_suffix_format() {
        // TODO: compatibility test, remove after v0.10.0
        assert_eq!(
            DateTimeAnchor::from_str("1d").unwrap(),
            DateTimeAnchor::InDays(1)
        );
        assert_eq!(
            DateTimeAnchor::from_str("2h").unwrap(),
            DateTimeAnchor::Relative(2 * 60 * 60)
        );
        assert_eq!(
            DateTimeAnchor::from_str("45m").unwrap(),
            DateTimeAnchor::Relative(45 * 60)
        );
        assert_eq!(
            DateTimeAnchor::from_str("1800s").unwrap(),
            DateTimeAnchor::Relative(1800)
        );
    }
}
