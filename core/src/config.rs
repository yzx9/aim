// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, TimeZone};
use serde::de;

use crate::Priority;

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
    pub default_due: Option<ConfigDue>,

    /// Default priority for new tasks.
    #[serde(default)]
    pub default_priority: Priority,

    /// If true, items with no priority will be listed first.
    #[serde(default)]
    pub default_priority_none_fist: bool,
}

impl Config {
    /// Normalize the configuration.
    pub fn normalize(&mut self) -> Result<(), Box<dyn Error>> {
        // Normalize calendar path
        self.calendar_path = expand_path(&self.calendar_path)?;

        // Normalize state directory
        match &self.state_dir {
            Some(a) => {
                self.state_dir = Some(
                    expand_path(a)
                        .map_err(|e| format!("Failed to expand state directory path: {e}"))?,
                )
            }

            None => match get_state_dir() {
                Ok(a) => self.state_dir = Some(a.join(APP_NAME)),
                Err(e) => log::warn!("Failed to get state directory: {e}"),
            },
        };

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConfigDue(Duration);

impl ConfigDue {
    pub fn datetime<Tz: TimeZone>(&self, now: DateTime<Tz>) -> DateTime<Tz> {
        now + self.0
    }
}

impl<'de> serde::Deserialize<'de> for ConfigDue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DueVisitor;

        impl<'de> de::Visitor<'de> for DueVisitor {
            type Value = ConfigDue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter
                    .write_str(r#"a duration string like "HH:MM", "1d", "24h", "60m", or "1800s""#)
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                parse_duration(value)
                    .map(ConfigDue)
                    .map_err(|e| de::Error::custom(e.to_string()))
            }
        }

        deserializer.deserialize_str(DueVisitor)
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

/// Parse a duration string in the format "HH:MM" / "1d" / "24h" / "60m" / "1800s".
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
