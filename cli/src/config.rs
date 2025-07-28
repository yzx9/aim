// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use aimcal_core::{Config as CoreConfig, Priority};
use chrono::Duration;
use colored::Colorize;
use std::{
    error::Error,
    path::{Path, PathBuf},
    str::FromStr,
};

pub const APP_NAME: &str = "aim";

#[derive(Debug, serde::Deserialize)]
struct ConfigRaw {
    calendar_path: PathBuf,

    state_dir: Option<PathBuf>,

    /// Default due time for new tasks, in the format "HH:MM" or "1d" / "24h" / "60m" / "1800s".
    default_due: Option<String>,

    /// Default priority for new tasks, support "high", "mid", "low", "none" or 0-9.
    default_priority: Option<Priority>,
}

/// Configuration for the Aim application.
#[derive(Debug)]
pub struct Config {
    /// Core configuration for the calendar.
    pub core: CoreConfig,

    /// Directory for storing application state.
    pub state_dir: Option<PathBuf>,

    /// Default priority for new tasks.
    pub default_priority: Priority,
}

impl TryFrom<ConfigRaw> for Config {
    type Error = Box<dyn Error>;

    fn try_from(raw: ConfigRaw) -> Result<Self, Self::Error> {
        let state_dir = match raw.state_dir {
            Some(a) => Some(
                expand_path(&a)
                    .map_err(|e| format!("Failed to expand state directory path: {e}"))?,
            ),
            None => match get_state_dir() {
                Ok(a) => Some(a.join(APP_NAME)),
                Err(e) => {
                    log::warn!("Failed to get state directory: {e}");
                    println!(
                        "{}",
                        "No state directory configured, some features not available.".red()
                    );
                    None
                }
            },
        };

        let default_due = raw
            .default_due
            .map(|a| parse_duration(&a))
            .transpose()
            .map_err(|e| format!("Failed to parse default due duration: {e}"))?;

        Ok(Self {
            core: CoreConfig {
                calendar_path: expand_path(&raw.calendar_path)?,
                default_due,
            },
            state_dir,
            default_priority: raw.default_priority.unwrap_or_default(),
        })
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

/// Parse a duration string in the format "HH:MM" / "1d" / "24h" / "60m" / "1800s" into a `chrono::Duration`.
fn parse_duration(s: &str) -> Result<Duration, Box<dyn Error>> {
    // Try to parse "HH:MM" format
    if let Some((h, m)) = s.split_once(':') {
        let hours: i64 = h.trim().parse()?;
        let minutes: i64 = m.trim().parse()?;
        Ok(Duration::minutes(hours * 60 + minutes))
    }
    // Match suffix-based formats
    else if let Some(rest) = s.strip_suffix("d") {
        let days: i64 = rest.trim().parse()?;
        Ok(Duration::days(days))
    } else if let Some(rest) = s.strip_suffix("h") {
        let hours: i64 = rest.trim().parse()?;
        Ok(Duration::hours(hours))
    } else if let Some(rest) = s.strip_suffix("m") {
        let minutes: i64 = rest.trim().parse()?;
        Ok(Duration::minutes(minutes))
    } else if let Some(rest) = s.strip_suffix("s") {
        let minutes: i64 = rest.trim().parse()?;
        Ok(Duration::seconds(minutes))
    } else {
        Err(format!("Invalid duration format: {s}").into())
    }
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

    #[test]
    fn test_parse_duration_colon_format() {
        let d = parse_duration("01:30").unwrap();
        assert_eq!(d, Duration::minutes(90));

        let d = parse_duration("00:00").unwrap();
        assert_eq!(d, Duration::minutes(0));

        let d = parse_duration("12:00").unwrap();
        assert_eq!(d, Duration::hours(12));
    }

    #[test]
    fn test_parse_duration_suffix_format() {
        assert_eq!(parse_duration("1d").unwrap(), Duration::days(1));
        assert_eq!(parse_duration("2h").unwrap(), Duration::hours(2));
        assert_eq!(parse_duration("45m").unwrap(), Duration::minutes(45));
        assert_eq!(parse_duration("1800s").unwrap(), Duration::seconds(1800));
    }

    #[test]
    fn test_parse_duration_invalid_format() {
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("99x").is_err());
        assert!(parse_duration("12:xx").is_err());
        assert!(parse_duration("12:").is_err());
        assert!(parse_duration("12").is_err());
    }
}
