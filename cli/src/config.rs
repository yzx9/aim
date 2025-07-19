// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use aim_core::Config;
use std::{error::Error, path::PathBuf};
use xdg::BaseDirectories;

pub const APP_NAME: &str = "aim";

/// Parse the configuration file.
pub async fn parse_config(path: Option<PathBuf>) -> Result<Config, Box<dyn Error>> {
    let path = match path {
        Some(path) if !path.is_absolute() => expand_path(path.to_str().unwrap())?,
        Some(path) => path.to_owned(),
        None => {
            // TODO: zero config should works
            let home = BaseDirectories::with_prefix(APP_NAME)
                .get_config_home()
                .ok_or("Failed to get user-specific config directory")?;

            let config = home.join("config.toml");
            if !config.exists() {
                return Err(format!("No config found at: {}", config.display()).into());
            }
            config
        }
    };

    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read config file: {e}"))?;

    let table: toml::Table = content
        .parse()
        .map_err(|e| format!("Failed to parse config: {e}"))?;

    let calendar_path = table
        .get("calendar_path")
        .and_then(|v| v.as_str())
        .ok_or("Missing or invalid 'calendar_path' in config")?;

    let calendar_path = expand_path(calendar_path)?;

    Ok(Config::new(calendar_path))
}

/// Handle tilde (~) and environment variables in the path
fn expand_path(path: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Handle tilde and home directory
    for prefix in ["~/", "$HOME/", "${HOME}/"] {
        if let Some(stripped) = path.strip_prefix(prefix) {
            let home = home::home_dir().ok_or("User-specific home directory not found")?;
            return Ok(home.join(stripped));
        }
    }

    // Handle XDG_CONFIG_HOME
    for prefix in ["$XDG_CONFIG_HOME/", "${XDG_CONFIG_HOME}/"] {
        if let Some(stripped) = path.strip_prefix(prefix) {
            let config_home = BaseDirectories::with_prefix(APP_NAME)
                .get_config_home()
                .ok_or("User-specific config directory not found")?;

            let app_name = APP_NAME.to_string() + "/";
            if let Some(rest) = stripped.strip_prefix(&app_name) {
                // If the path starts with app name, we assume it's relative to the config home
                return Ok(config_home.join(rest));
            }

            return Ok(config_home.join(stripped));
        }
    }

    Ok(path.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_path_tilde() {
        let home = home::home_dir().unwrap();
        let result = expand_path("~/Documents").unwrap();
        assert!(result.is_absolute());
        assert!(result.to_string_lossy().ends_with("/Documents"));
        assert_eq!(result, home.join("Documents"));
    }

    #[test]
    fn test_expand_path_home_env() {
        let home = home::home_dir().unwrap();
        for prefix in ["~", "$HOME", "${HOME}"] {
            let result = expand_path(&format!("{prefix}/Documents")).unwrap();
            assert!(result.is_absolute());
            assert!(result.to_string_lossy().ends_with("/Documents"));
            assert_eq!(result, home.join("Documents"));
        }
    }

    #[test]
    fn test_expand_path_xdg_config() {
        let config_home = BaseDirectories::with_prefix(APP_NAME)
            .get_config_home()
            .unwrap();

        for prefix in ["$XDG_CONFIG_HOME", "${XDG_CONFIG_HOME}"] {
            let result = expand_path(&format!("{prefix}/{APP_NAME}/config.toml")).unwrap();
            assert!(result.is_absolute());
            assert!(result.to_string_lossy().ends_with("/config.toml"));
            assert_eq!(result, config_home.join("config.toml"));
        }
    }

    #[test]
    fn test_expand_path_absolute() {
        let absolute_path = "/etc/passwd";
        let result = expand_path(absolute_path).unwrap();
        assert_eq!(result, PathBuf::from(absolute_path));
    }

    #[test]
    fn test_expand_path_relative() {
        let relative_path = "relative/path/to/file";
        let result = expand_path(relative_path).unwrap();
        assert_eq!(result, PathBuf::from(relative_path));
    }
}
