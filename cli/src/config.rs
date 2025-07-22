// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use aim_core::Config as CoreConfig;
use serde::Deserialize;
use std::{
    error::Error,
    path::{Path, PathBuf},
    str::FromStr,
};

pub const APP_NAME: &str = "aim";

#[derive(Debug, Deserialize)]
struct ConfigRaw {
    calendar_path: PathBuf,
    state_dir: Option<PathBuf>,
}

#[derive(Debug)]
pub struct Config {
    pub core: CoreConfig,
    pub state_dir: Option<PathBuf>,
}

impl TryFrom<ConfigRaw> for Config {
    type Error = Box<dyn Error>;

    fn try_from(raw: ConfigRaw) -> Result<Self, Self::Error> {
        let core = CoreConfig {
            calendar_path: expand_path(&raw.calendar_path)?,
        };
        let state_dir = match raw.state_dir {
            Some(a) => Some(expand_path(&a)?.join(APP_NAME)),
            None => get_state_dir().ok(),
        };
        Ok(Self { core, state_dir })
    }
}

impl FromStr for Config {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str::<ConfigRaw>(s)?.try_into()
    }
}

impl Config {
    /// Parse the configuration file.
    pub async fn parse(path: Option<PathBuf>) -> Result<Config, Box<dyn Error>> {
        let path = match path {
            Some(path) => expand_path(&path)?,
            None => {
                // TODO: zero config should works
                // TODO: search config in multiple locations
                let config = get_config_dir()?.join(format!("{APP_NAME}/config.toml"));
                if !config.exists() {
                    return Err(format!("No config found at: {}", config.display()).into());
                }
                config
            }
        };

        tokio::fs::read_to_string(path)
            .await
            .map_err(|e| format!("Failed to read config file: {e}"))?
            .parse()
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
    dirs::home_dir().ok_or("User-specific home directory not found".into())
}

fn get_config_dir() -> Result<PathBuf, Box<dyn Error>> {
    #[cfg(unix)]
    let config_dir = xdg::BaseDirectories::new().get_config_home();
    #[cfg(windows)]
    let config_dir = dirs::config_dir();
    config_dir.ok_or("User-specific home directory not found".into())
}

fn get_state_dir() -> Result<PathBuf, Box<dyn Error>> {
    #[cfg(unix)]
    let state_dir = xdg::BaseDirectories::new().get_state_home();
    #[cfg(windows)]
    let state_dir = dirs::data_dir();
    state_dir.ok_or("User-specific state directory not found".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_path_home_env() {
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
    fn test_expand_path_config() {
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
    fn test_expand_path_absolute() {
        let absolute_path = PathBuf::from("/etc/passwd");
        let result = expand_path(&absolute_path).unwrap();
        assert_eq!(result, absolute_path);
    }

    #[test]
    fn test_expand_path_relative() {
        let relative_path = PathBuf::from("relative/path/to/file");
        let result = expand_path(&relative_path).unwrap();
        assert_eq!(result, relative_path);
    }
}
