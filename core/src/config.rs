// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

#![allow(
    clippy::indexing_slicing,
    clippy::format_push_string,
    clippy::missing_panics_doc
)]

use std::error::Error;
use std::path::{Path, PathBuf};

use crate::{DateTimeAnchor, Priority};
use aimcal_caldav::AuthMethod;

/// The name of the AIM application.
pub const APP_NAME: &str = "aim";

/// Backend configuration for storage.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type")]
pub enum BackendConfig {
    /// Local ICS file backend.
    #[serde(rename = "local")]
    Local {
        /// Path to the calendar directory for ICS files.
        calendar_path: Option<String>,
    },
    /// `CalDAV` server backend.
    #[serde(rename = "caldav")]
    Caldav {
        /// Base URL of the `CalDAV` server.
        base_url: String,
        /// Calendar home path on the server.
        calendar_home: String,
        /// Href of the calendar collection on the server.
        calendar_href: String,
        /// Authentication method.
        auth: AuthMethod,
        /// Request timeout in seconds.
        #[serde(default = "default_timeout_secs")]
        timeout_secs: u64,
        /// User agent string for HTTP requests.
        #[serde(default = "default_user_agent")]
        user_agent: String,
    },
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self::Local {
            calendar_path: None,
        }
    }
}

fn default_timeout_secs() -> u64 {
    30
}

fn default_user_agent() -> String {
    "aimcal/0.11.0".to_string()
}

/// Configuration for a single calendar in the TOML file.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "kind")]
pub enum CalendarConfig {
    /// Local ICS file calendar.
    #[serde(rename = "local")]
    Local {
        /// Path to the calendar directory for ICS files.
        #[serde(default)]
        calendar_path: Option<String>,
    },
    /// `CalDAV` server calendar.
    #[serde(rename = "caldav")]
    Caldav {
        /// Base URL of the `CalDAV` server.
        base_url: String,
        /// Calendar home path on the server.
        calendar_home: String,
        /// Href of the calendar collection on the server.
        calendar_href: String,
        /// Authentication method.
        auth: AuthMethod,
        /// Request timeout in seconds.
        #[serde(default = "default_timeout_secs")]
        timeout_secs: u64,
        /// User agent string for HTTP requests.
        #[serde(default = "default_user_agent")]
        user_agent: String,
    },
}

/// Calendar entry in the TOML configuration.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CalendarEntry {
    /// Unique calendar identifier.
    pub id: String,
    /// Display name for the calendar.
    pub name: String,
    /// Calendar configuration.
    #[serde(flatten)]
    pub config: CalendarConfig,
    /// Priority for conflict resolution and display ordering.
    #[serde(default)]
    pub priority: i32,
    /// Whether the calendar is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

/// Configuration for the AIM application.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    /// Path to the calendar directory (optional ICS export/import).
    ///
    /// Deprecated: Use `calendars` array instead.
    ///
    /// If set, AIM will sync events/todos with ICS files in this directory.
    /// If not set, AIM will only use the `Db` for storage.
    #[serde(default)]
    pub calendar_path: Option<PathBuf>,

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

    /// Parent directory of the config file.
    ///
    /// Set by the CLI layer after parsing. Used by `normalize()` to resolve
    /// relative paths against the config file location rather than the CWD.
    #[serde(skip)]
    pub config_dir: Option<PathBuf>,

    /// Whether development mode is active.
    ///
    /// When true, `normalize()` enforces explicit `state_dir` configuration
    /// to prevent accidental use of the system default state directory.
    #[serde(skip)]
    pub dev_mode: bool,

    /// Backend configuration (deprecated, legacy single-backend).
    ///
    /// Deprecated: Use `calendars` array instead.
    #[serde(default)]
    pub backend: BackendConfig,

    /// Multi-calendar configuration.
    ///
    /// If present, overrides `backend` and `calendar_path`.
    #[serde(default)]
    pub calendars: Vec<CalendarEntry>,

    /// Default calendar ID for new items.
    #[serde(default = "default_calendar_id")]
    pub default_calendar: String,
}

fn default_calendar_id() -> String {
    "default".to_string()
}

impl Config {
    /// Normalize the configuration.
    ///
    /// # Errors
    /// If path normalization fails.
    #[tracing::instrument(skip(self))]
    pub fn normalize(&mut self) -> Result<(), Box<dyn Error>> {
        let config_parent = self.config_dir.as_deref();

        // Normalize calendar path if set (legacy format)
        if let Some(ref calendar_path) = self.calendar_path {
            self.calendar_path = Some(expand_path(calendar_path, config_parent)?);
        }

        // Normalize state directory
        if let Some(a) = &self.state_dir {
            let state_dir = expand_path(a, config_parent)
                .map_err(|e| format!("Failed to expand state directory path: {e}"))?;
            self.state_dir = Some(state_dir);
        } else {
            if self.dev_mode {
                return Err(
                    "Development mode requires state_dir to be explicitly configured".into(),
                );
            }
            match get_state_dir() {
                Ok(a) => self.state_dir = Some(a.join(APP_NAME)),
                Err(err) => tracing::warn!(err, "failed to get state directory"),
            }
        }

        // Normalize calendar paths for multi-calendar configuration
        for i in 0..self.calendars.len() {
            let calendar = self.calendars.get(i).unwrap();
            self.calendars[i] = CalendarEntry {
                id: calendar.id.clone(),
                name: calendar.name.clone(),
                config: match &calendar.config {
                    CalendarConfig::Local { calendar_path, .. } => CalendarConfig::Local {
                        calendar_path: if let Some(path) = calendar_path {
                            let p = expand_path(&PathBuf::from(path), None)
                                .map_err(|e| {
                                    format!(
                                        "Failed to expand calendar path for {}: {e}",
                                        calendar.id
                                    )
                                })?
                                .to_string_lossy()
                                .to_string();
                            Some(p)
                        } else if let Some(ref state_dir) = self.state_dir {
                            let p = state_dir
                                .join("calendar")
                                .join(&calendar.id)
                                .to_string_lossy()
                                .to_string();
                            Some(p)
                        } else {
                            calendar_path.clone()
                        },
                    },
                    CalendarConfig::Caldav { .. } => calendar.config.clone(),
                },
                priority: calendar.priority,
                enabled: calendar.enabled,
            };
        }

        Ok(())
    }

    /// Check if the configuration uses legacy single-calendar format.
    ///
    /// Returns `true` if `calendars` array is empty (legacy or default mode).
    #[must_use]
    pub fn is_legacy_format(&self) -> bool {
        // If calendars array is present and not empty, it's not legacy
        self.calendars.is_empty()
    }

    /// Generate a warning message for legacy configuration.
    ///
    /// This provides helpful guidance to users about migrating to multi-calendar format.
    #[must_use]
    pub fn legacy_warning(&self) -> String {
        if !self.calendars.is_empty() {
            return String::new();
        }

        let mut warning =
            String::from("Warning: Using legacy single-calendar configuration format.\n\n");

        if self.calendar_path.is_some() {
            warning += "Legacy 'calendar_path' is detected.\n";
        }

        if !matches!(
            &self.backend,
            BackendConfig::Local {
                calendar_path: None
            }
        ) {
            warning += "Legacy 'backend' configuration is detected.\n";
        }

        warning +=
            "The multi-calendar feature requires updating to the 'calendars' array format.\n\n";
        warning += "Please update your aim.toml to use the following format:\n\n";
        warning += "[[core]\n";
        warning += "[[calendars]]\n";
        warning += "id = \"default\"\n";
        warning += "name = \"Default\"\n";
        warning += "kind = \"local\"\n";
        warning += "priority = 0\n";
        warning += "enabled = true\n";

        if let Some(ref path) = self.calendar_path {
            warning.push_str("\n# Your existing path can be used as:\n");
            warning.push_str(&format!("calendar_path = \"{}\"\n", path.display()));
        }

        warning += "\nSee documentation for full migration guide.\n";
        warning
    }

    /// Get all enabled calendars ordered by priority.
    #[must_use]
    pub fn enabled_calendars(&self) -> Vec<&CalendarEntry> {
        let mut calendars: Vec<_> = self.calendars.iter().filter(|c| c.enabled).collect();
        calendars.sort_by_key(|c| c.priority);
        calendars
    }

    /// Get the configuration for a specific calendar by ID.
    #[must_use]
    pub fn get_calendar_config(&self, id: &str) -> Option<&CalendarConfig> {
        self.calendars
            .iter()
            .find(|c| c.id == id)
            .map(|c| &c.config)
    }
}

/// Handle tilde (~) and environment variables in the path.
///
/// Relative paths that don't match any special prefix are resolved against
/// `config_parent` when provided, or returned as-is otherwise.
fn expand_path(path: &Path, config_parent: Option<&Path>) -> Result<PathBuf, Box<dyn Error>> {
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
        &[r"%LOCALAPPDATA%\", "%LOCALAPPDATA%"]
    };

    for prefix in config_prefixes {
        if let Some(stripped) = path.strip_prefix(prefix) {
            return Ok(get_config_dir()?.join(stripped));
        }
    }

    match config_parent {
        Some(parent) => Ok(parent.join(path)),
        None => Ok(path.into()),
    }
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
        assert_eq!(config.calendar_path, Some(PathBuf::from("calendar")));
        assert_eq!(config.state_dir, Some(PathBuf::from("state")));
        assert_eq!(config.default_due, Some(DateTimeAnchor::InDays(1)));
        assert_eq!(config.default_priority, Priority::P2);
        assert!(config.default_priority_none_fist);
    }

    #[test]
    #[allow(clippy::needless_raw_string_hashes)]
    fn parses_minimal_toml_with_defaults() {
        const TOML: &str = r#"
"#;

        let config: Config = toml::from_str(TOML).expect("Failed to parse TOML");
        assert_eq!(config.calendar_path, None);
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
            &[r"~", r"%UserProfile%", r"%UserProfile%"]
        };

        for prefix in home_prefixes {
            let result = expand_path(&PathBuf::from(format!("{prefix}/Documents")), None).unwrap();
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
            &[r"%LOCALAPPDATA%", "%LOCALAPPDATA%"]
        };

        for prefix in config_prefixes {
            let result =
                expand_path(&PathBuf::from(format!("{prefix}/config.toml")), None).unwrap();
            assert_eq!(result, config_dir.join("config.toml"));
            assert!(result.is_absolute());
        }
    }

    #[test]
    fn preserves_absolute_path() {
        let absolute_path = PathBuf::from("/etc/passwd");
        let result = expand_path(&absolute_path, None).unwrap();
        assert_eq!(result, absolute_path);
    }

    #[test]
    fn preserves_relative_path_without_config_parent() {
        let relative_path = PathBuf::from("relative/path/to/file");
        let result = expand_path(&relative_path, None).unwrap();
        assert_eq!(result, relative_path);
    }

    #[test]
    fn resolves_relative_path_against_config_parent() {
        let relative_path = PathBuf::from("relative/path/to/file");
        let config_parent = PathBuf::from("/etc/aim");

        let result = expand_path(&relative_path, Some(&config_parent)).unwrap();
        assert_eq!(result, PathBuf::from("/etc/aim/relative/path/to/file"));
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

    #[test]
    fn parses_multi_calendar_config() {
        const TOML: &str = r#"
default_calendar = "personal"

[[calendars]]
id = "personal"
name = "Personal"
kind = "local"
priority = 0
enabled = true
calendar_path = "~/personal"

[[calendars]]
id = "work"
name = "Work"
kind = "caldav"
priority = 1
enabled = true
base_url = "https://caldav.example.com"
calendar_home = "/dav/calendars/user/"
calendar_href = "/dav/calendars/user/work/"
auth = { type = "basic", username = "user", password = "pass" }
timeout_secs = 30
user_agent = "aimcal/0.11.0"
"#;

        let config: Config = toml::from_str(TOML).expect("Failed to parse TOML");
        assert_eq!(config.calendars.len(), 2);

        assert_eq!(config.calendars[0].id, "personal");
        assert_eq!(config.calendars[0].name, "Personal");
        assert_eq!(config.calendars[0].priority, 0);
        assert!(config.calendars[0].enabled);
        assert!(matches!(
            &config.calendars[0].config,
            CalendarConfig::Local { .. }
        ));

        assert_eq!(config.calendars[1].id, "work");
        assert_eq!(config.calendars[1].name, "Work");
        assert_eq!(config.calendars[1].priority, 1);
        assert!(config.calendars[1].enabled);
        assert!(matches!(
            &config.calendars[1].config,
            CalendarConfig::Caldav { .. }
        ));

        assert_eq!(config.default_calendar, "personal");
    }

    #[test]
    fn is_legacy_format_detects_legacy_config() {
        const TOML: &str = r#"
calendar_path = "calendar"
state_dir = "state"
"#;

        let config: Config = toml::from_str(TOML).expect("Failed to parse TOML");
        assert!(config.is_legacy_format());
    }

    #[test]
    fn is_legacy_format_returns_false_for_multi_calendar() {
        const TOML: &str = r#"
[core]
[[calendars]]
id = "personal"
name = "Personal"
kind = "local"
"#;

        let config: Config = toml::from_str(TOML).expect("Failed to parse TOML");
        assert!(!config.is_legacy_format());
    }

    #[test]
    fn legacy_warning_provides_helpful_message() {
        const TOML: &str = r#"
calendar_path = "calendar"
"#;

        let config: Config = toml::from_str(TOML).expect("Failed to parse TOML");
        let warning = config.legacy_warning();

        assert!(warning.contains("Warning: Using legacy single-calendar configuration"));
        assert!(warning.contains("calendars"));
        assert!(warning.contains("id = \"default\""));
    }

    #[test]
    fn enabled_calendars_returns_enabled_only() {
        const TOML: &str = r#"
[[calendars]]
id = "personal"
name = "Personal"
kind = "local"
enabled = true

[[calendars]]
id = "work"
name = "Work"
kind = "local"
enabled = false

[[calendars]]
id = "archive"
name = "Archive"
kind = "local"
enabled = true
priority = 5
"#;

        let config: Config = toml::from_str(TOML).expect("Failed to parse TOML");
        let enabled = config.enabled_calendars();

        assert_eq!(enabled.len(), 2);
        assert!(
            enabled
                .iter()
                .all(|c| c.id == "personal" || c.id == "archive")
        );
        assert!(!enabled.iter().any(|c| c.id == "work"));
    }

    #[test]
    fn get_calendar_config_returns_config_by_id() {
        const TOML: &str = r#"
[[calendars]]
id = "personal"
name = "Personal"
kind = "local"
calendar_path = "~/personal"
"#;

        let config: Config = toml::from_str(TOML).expect("Failed to parse TOML");
        let personal_config = config.get_calendar_config("personal");

        assert!(personal_config.is_some());

        if let Some(CalendarConfig::Local { calendar_path }) = personal_config {
            assert!(calendar_path.as_deref() == Some("~/personal"));
        }

        let work_config = config.get_calendar_config("work");
        assert!(work_config.is_none());
    }
}
