// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

#![allow(
    clippy::indexing_slicing,
    clippy::format_push_string,
    clippy::missing_panics_doc
)]

use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use crate::{DateTimeAnchor, Priority};
use aimcal_caldav::AuthMethod;

/// The name of the AIM application.
pub const APP_NAME: &str = "aim";

fn default_timeout_secs() -> u64 {
    30
}

fn default_user_agent() -> String {
    "aimcal/0.11.0".to_string()
}

/// Store definition for shared connection configuration.
///
/// Stores define how to connect to a calendar storage. Multiple calendars
/// can reference the same store, avoiding duplication of connection details.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type")]
pub enum StoreDef {
    /// Local ICS file store.
    #[serde(rename = "local")]
    Local {
        /// Path to the calendar directory for ICS files.
        calendar_path: Option<String>,
    },
    /// `CalDAV` server store.
    #[serde(rename = "caldav")]
    Caldav {
        /// Base URL of the `CalDAV` server.
        base_url: String,
        /// Calendar home path on the server.
        calendar_home: String,
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
///
/// Each calendar references a store by ID and provides calendar-specific fields
/// such as the calendar href (for `CalDAV`) or an optional path override (for local).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CalendarEntry {
    /// Unique calendar identifier.
    pub id: String,
    /// Display name for the calendar.
    pub name: String,
    /// Reference to a store definition in `stores`.
    pub store: String,
    /// Href of the calendar collection on the server (required for caldav stores).
    pub calendar_href: Option<String>,
    /// Path to the calendar directory (optional override for local backends).
    pub calendar_path: Option<String>,
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

    /// Store definitions — shared connection configurations keyed by name.
    #[serde(default)]
    pub stores: HashMap<String, StoreDef>,

    /// Multi-calendar configuration.
    ///
    /// If present, each entry references a store from `stores`.
    #[serde(default)]
    pub calendars: Vec<CalendarEntry>,

    /// Default calendar ID for new items.
    #[serde(default = "default_calendar_id")]
    pub default_calendar: String,

    /// Paths to files containing `KEY=VALUE` pairs for secret lookup.
    ///
    /// Paths are resolved relative to `config_dir`. Variables from these
    /// files are accessible via `${ENV:VAR_NAME}` syntax, with lookup
    /// priority: secrets file values first, then actual environment variables.
    #[serde(default)]
    pub secrets_files: Vec<String>,
}

fn default_calendar_id() -> String {
    "default".to_string()
}

impl Config {
    /// Expand `${ENV:VAR_NAME}` references in all string fields.
    ///
    /// Loads secrets files first, then walks every string field and replaces
    /// `${ENV:...}` references with resolved values. Lookup order: secrets
    /// file values first, then actual environment variables.
    ///
    /// # Errors
    /// Returns an error if a referenced variable is not found.
    pub fn expand_env_vars(&mut self) -> Result<(), Box<dyn Error>> {
        let secrets = load_secrets_files(&self.secrets_files, self.config_dir.as_deref())?;

        for store_def in self.stores.values_mut() {
            match store_def {
                StoreDef::Local { calendar_path } => {
                    if let Some(path) = calendar_path.take() {
                        *calendar_path = Some(expand_env_var(&path, &secrets)?);
                    }
                }
                StoreDef::Caldav {
                    base_url,
                    calendar_home,
                    auth,
                    user_agent,
                    ..
                } => {
                    *base_url = expand_env_var(base_url, &secrets)?;
                    *calendar_home = expand_env_var(calendar_home, &secrets)?;
                    *user_agent = expand_env_var(user_agent, &secrets)?;
                    match auth {
                        AuthMethod::None => {}
                        AuthMethod::Basic { username, password } => {
                            *username = expand_env_var(username, &secrets)?;
                            *password = expand_env_var(password, &secrets)?;
                        }
                        AuthMethod::Bearer { token } => {
                            *token = expand_env_var(token, &secrets)?;
                        }
                    }
                }
            }
        }

        for calendar in &mut self.calendars {
            if let Some(ref href) = calendar.calendar_href {
                calendar.calendar_href = Some(expand_env_var(href, &secrets)?);
            }
            if let Some(ref path) = calendar.calendar_path {
                calendar.calendar_path = Some(expand_env_var(path, &secrets)?);
            }
        }

        Ok(())
    }

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
            let store_def = self.stores.get(&calendar.store);

            // Only resolve calendar_path for local stores
            let calendar_path =
                if matches!(store_def, Some(StoreDef::Local { .. })) || calendar.store == "local" {
                    if let Some(ref path) = calendar.calendar_path {
                        let p = expand_path(&PathBuf::from(path), None)
                            .map_err(|e| {
                                format!("Failed to expand calendar path for {}: {e}", calendar.id)
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
                        calendar.calendar_path.clone()
                    }
                } else {
                    calendar.calendar_path.clone()
                };

            self.calendars[i] = CalendarEntry {
                id: calendar.id.clone(),
                name: calendar.name.clone(),
                store: calendar.store.clone(),
                calendar_href: calendar.calendar_href.clone(),
                calendar_path,
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

        warning +=
            "The multi-calendar feature requires updating to the 'calendars' array format.\n\n";
        warning += "Please update your aim.toml to use the following format:\n\n";
        warning += "[stores.local]\n";
        warning += "type = \"local\"\n\n";
        warning += "[[calendars]]\n";
        warning += "id = \"default\"\n";
        warning += "name = \"Default\"\n";
        warning += "store = \"local\"\n";
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

    /// Resolve the store definition for a calendar entry.
    ///
    /// Returns `None` if the calendar is not found or the store reference is invalid.
    #[must_use]
    pub fn resolve_store(&self, calendar_id: &str) -> Option<(&CalendarEntry, &StoreDef)> {
        let entry = self.calendars.iter().find(|c| c.id == calendar_id)?;
        let store = self.stores.get(&entry.store)?;
        Some((entry, store))
    }

    /// Get a specific calendar entry by ID.
    #[must_use]
    pub fn get_calendar(&self, id: &str) -> Option<&CalendarEntry> {
        self.calendars.iter().find(|c| c.id == id)
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

/// Expand `${ENV:VAR_NAME}` references in a string.
///
/// Lookup order: `secrets` map first, then `std::env::var`.
///
/// # Errors
/// Returns an error if a referenced variable is not found in either source.
fn expand_env_var(
    input: &str,
    secrets: &HashMap<String, String>,
) -> Result<String, Box<dyn Error>> {
    use regex::Regex;

    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"\$\{ENV:([A-Za-z_][A-Za-z0-9_]*)\}").unwrap());

    let mut result = String::with_capacity(input.len());
    let mut last_end = 0;

    for cap in re.captures_iter(input) {
        let m = cap.get(0).unwrap();
        let var_name = cap.get(1).unwrap().as_str();

        let value = secrets
            .get(var_name)
            .cloned()
            .or_else(|| std::env::var(var_name).ok())
            .ok_or_else(|| {
                format!("Variable '{var_name}' not found (not in secrets files or environment)")
            })?;

        result.push_str(&input[last_end..m.start()]);
        result.push_str(&value);
        last_end = m.end();
    }

    result.push_str(&input[last_end..]);
    Ok(result)
}

/// Parse `KEY=VALUE` content from a secrets file.
///
/// Lines starting with `#` are comments. Empty lines are ignored.
fn parse_secrets_content(
    contents: &str,
    path: &Path,
    secrets: &mut HashMap<String, String>,
) -> Result<(), Box<dyn Error>> {
    for (line_num, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            if key.is_empty() {
                return Err(format!(
                    "Empty key in secrets file '{}' at line {}",
                    path.display(),
                    line_num + 1
                )
                .into());
            }
            secrets.insert(key, value);
        } else {
            return Err(format!(
                "Invalid line in secrets file '{}' at line {}: expected KEY=VALUE, got '{}'",
                path.display(),
                line_num + 1,
                trimmed
            )
            .into());
        }
    }
    Ok(())
}

/// Load `KEY=VALUE` pairs from all configured secrets files.
///
/// Files are loaded in order; later files override earlier ones.
/// Paths are resolved relative to `config_dir` when available.
fn load_secrets_files(
    secrets_files: &[String],
    config_dir: Option<&Path>,
) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut secrets = HashMap::new();
    for file_path_str in secrets_files {
        let path = if Path::new(file_path_str).is_absolute() {
            PathBuf::from(file_path_str)
        } else if let Some(dir) = config_dir {
            dir.join(file_path_str)
        } else {
            PathBuf::from(file_path_str)
        };

        if !path.exists() {
            tracing::warn!(path = %path.display(), "secrets file not found, skipping");
            continue;
        }
        let contents = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read secrets file '{}': {e}", path.display()))?;
        parse_secrets_content(&contents, &path, &mut secrets)?;
    }
    Ok(secrets)
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

[stores.mylocal]
type = "local"

[stores.radicale]
type = "caldav"
base_url = "https://caldav.example.com"
calendar_home = "/dav/calendars/user/"
auth = { type = "basic", username = "user", password = "pass" }
timeout_secs = 30
user_agent = "aimcal/0.11.0"

[[calendars]]
id = "personal"
name = "Personal"
store ="mylocal"
priority = 0
enabled = true
calendar_path = "~/personal"

[[calendars]]
id = "work"
name = "Work"
store ="radicale"
priority = 1
enabled = true
calendar_href = "/dav/calendars/user/work/"
"#;

        let config: Config = toml::from_str(TOML).expect("Failed to parse TOML");
        assert_eq!(config.calendars.len(), 2);
        assert_eq!(config.stores.len(), 2);

        assert_eq!(config.calendars[0].id, "personal");
        assert_eq!(config.calendars[0].name, "Personal");
        assert_eq!(config.calendars[0].store, "mylocal");
        assert_eq!(config.calendars[0].priority, 0);
        assert!(config.calendars[0].enabled);

        assert_eq!(config.calendars[1].id, "work");
        assert_eq!(config.calendars[1].name, "Work");
        assert_eq!(config.calendars[1].store, "radicale");
        assert_eq!(
            config.calendars[1].calendar_href,
            Some("/dav/calendars/user/work/".to_string())
        );
        assert_eq!(config.calendars[1].priority, 1);
        assert!(config.calendars[1].enabled);

        assert_eq!(config.default_calendar, "personal");

        // Verify backends
        assert!(matches!(
            config.stores.get("mylocal"),
            Some(StoreDef::Local { .. })
        ));
        assert!(matches!(
            config.stores.get("radicale"),
            Some(StoreDef::Caldav { .. })
        ));
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
[stores.local]
type = "local"

[[calendars]]
id = "personal"
name = "Personal"
store ="local"
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
[stores.local]
type = "local"

[[calendars]]
id = "personal"
name = "Personal"
store ="local"
enabled = true

[[calendars]]
id = "work"
name = "Work"
store ="local"
enabled = false

[[calendars]]
id = "archive"
name = "Archive"
store ="local"
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
    fn resolve_store_returns_entry_and_def() {
        const TOML: &str = r#"
[stores.radicale]
type = "caldav"
base_url = "https://caldav.example.com"
calendar_home = "/dav/"
auth = { type = "basic", username = "u", password = "p" }

[[calendars]]
id = "work"
name = "Work"
store ="radicale"
calendar_href = "/dav/work/"
"#;

        let config: Config = toml::from_str(TOML).expect("Failed to parse TOML");
        let (entry, backend) = config.resolve_store("work").unwrap();
        assert_eq!(entry.id, "work");
        assert!(matches!(backend, StoreDef::Caldav { .. }));
        assert!(config.resolve_store("nonexistent").is_none());
    }

    #[test]
    fn get_calendar_returns_entry_by_id() {
        const TOML: &str = r#"
[stores.local]
type = "local"

[[calendars]]
id = "personal"
name = "Personal"
store ="local"
calendar_path = "~/personal"
"#;

        let config: Config = toml::from_str(TOML).expect("Failed to parse TOML");
        let personal = config.get_calendar("personal");
        assert!(personal.is_some());
        assert_eq!(
            personal.unwrap().calendar_path.as_deref(),
            Some("~/personal")
        );

        assert!(config.get_calendar("work").is_none());
    }

    #[test]
    fn multiple_calendars_share_backend() {
        const TOML: &str = r#"
default_calendar = "home"

[stores.radicale]
type = "caldav"
base_url = "https://caldav.example.com/"
calendar_home = "/user/"
auth = { type = "basic", username = "u", password = "p" }

[[calendars]]
id = "home"
name = "Home"
store ="radicale"
calendar_href = "/user/home/"
priority = 0

[[calendars]]
id = "work"
name = "Work"
store ="radicale"
calendar_href = "/user/work/"
priority = 1

[[calendars]]
id = "test"
name = "Test"
store ="radicale"
calendar_href = "/user/test/"
priority = 2
"#;

        let config: Config = toml::from_str(TOML).expect("Failed to parse TOML");
        assert_eq!(config.calendars.len(), 3);
        assert_eq!(config.stores.len(), 1);

        // All calendars reference the same store
        for calendar in &config.calendars {
            assert_eq!(calendar.store, "radicale");
        }
    }

    #[test]
    fn expand_env_var_no_placeholders() {
        let secrets = HashMap::new();
        let result = expand_env_var("hello world", &secrets).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn expand_env_var_simple_replacement() {
        let mut secrets = HashMap::new();
        secrets.insert("FOO".to_string(), "bar".to_string());
        let result = expand_env_var("${ENV:FOO}", &secrets).unwrap();
        assert_eq!(result, "bar");
    }

    #[test]
    fn expand_env_var_embedded_in_string() {
        let mut secrets = HashMap::new();
        secrets.insert("HOST".to_string(), "example.com".to_string());
        let result = expand_env_var("https://${ENV:HOST}/dav", &secrets).unwrap();
        assert_eq!(result, "https://example.com/dav");
    }

    #[test]
    fn expand_env_var_multiple_occurrences() {
        let mut secrets = HashMap::new();
        secrets.insert("A".to_string(), "x".to_string());
        secrets.insert("B".to_string(), "y".to_string());
        let result = expand_env_var("${ENV:A}/${ENV:B}", &secrets).unwrap();
        assert_eq!(result, "x/y");
    }

    #[test]
    fn expand_env_var_missing_variable() {
        let secrets = HashMap::new();
        let result = expand_env_var("${ENV:NONEXISTENT_VAR}", &secrets);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("NONEXISTENT_VAR"));
    }

    #[test]
    fn expand_env_var_secrets_priority_over_env() {
        let mut secrets = HashMap::new();
        secrets.insert("TEST_PRIORITY_VAR".to_string(), "from_secrets".to_string());
        // Note: this test assumes TEST_PRIORITY_VAR is not set in env,
        // but the secrets map takes priority anyway
        let result = expand_env_var("${ENV:TEST_PRIORITY_VAR}", &secrets).unwrap();
        assert_eq!(result, "from_secrets");
    }

    #[test]
    fn parse_secrets_simple() {
        let mut secrets = HashMap::new();
        let path = Path::new("test.env");
        parse_secrets_content("KEY=value\nOTHER=123", path, &mut secrets).unwrap();
        assert_eq!(secrets.get("KEY").unwrap(), "value");
        assert_eq!(secrets.get("OTHER").unwrap(), "123");
    }

    #[test]
    fn parse_secrets_comments_and_blanks() {
        let mut secrets = HashMap::new();
        let path = Path::new("test.env");
        parse_secrets_content(
            "# comment\n\nKEY=value\n# another comment\n",
            path,
            &mut secrets,
        )
        .unwrap();
        assert_eq!(secrets.len(), 1);
        assert_eq!(secrets.get("KEY").unwrap(), "value");
    }

    #[test]
    fn parse_secrets_trimming() {
        let mut secrets = HashMap::new();
        let path = Path::new("test.env");
        parse_secrets_content("  KEY  =  value  ", path, &mut secrets).unwrap();
        assert_eq!(secrets.get("KEY").unwrap(), "value");
    }

    #[test]
    fn parse_secrets_value_with_equals() {
        let mut secrets = HashMap::new();
        let path = Path::new("test.env");
        parse_secrets_content("KEY=a=b", path, &mut secrets).unwrap();
        assert_eq!(secrets.get("KEY").unwrap(), "a=b");
    }

    #[test]
    fn parse_secrets_invalid_line() {
        let mut secrets = HashMap::new();
        let path = Path::new("test.env");
        let result = parse_secrets_content("no_equals_here", path, &mut secrets);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("expected KEY=VALUE")
        );
    }

    #[test]
    fn parse_secrets_empty_key() {
        let mut secrets = HashMap::new();
        let path = Path::new("test.env");
        let result = parse_secrets_content("=value", path, &mut secrets);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty key"));
    }

    #[test]
    fn config_expand_env_vars_in_caldav_store() {
        const TOML: &str = r#"
[stores.mycaldav]
type = "caldav"
base_url = "https://${ENV:HOST}/dav"
calendar_home = "/dav/${ENV:USER}/"
auth = { type = "basic", username = "${ENV:USER}", password = "${ENV:PASS}" }

[[calendars]]
id = "work"
name = "Work"
store = "mycaldav"
calendar_href = "/dav/${ENV:USER}/work/"
"#;

        let config: Config = toml::from_str(TOML).expect("Failed to parse TOML");

        let mut secrets = HashMap::new();
        secrets.insert("HOST".to_string(), "caldav.example.com".to_string());
        secrets.insert("USER".to_string(), "admin".to_string());
        secrets.insert("PASS".to_string(), "s3cret".to_string());

        // Use load_secrets_files indirectly by calling expand_env_vars
        // We need to set env vars since we can't inject secrets directly
        // Instead, test the expand_env_var function directly on the config fields
        let store = config.stores.get("mycaldav").unwrap();
        match store {
            StoreDef::Caldav {
                base_url,
                calendar_home,
                auth,
                ..
            } => {
                assert_eq!(
                    expand_env_var(base_url, &secrets).unwrap(),
                    "https://caldav.example.com/dav"
                );
                assert_eq!(
                    expand_env_var(calendar_home, &secrets).unwrap(),
                    "/dav/admin/"
                );
                match auth {
                    AuthMethod::Basic { username, password } => {
                        assert_eq!(expand_env_var(username, &secrets).unwrap(), "admin");
                        assert_eq!(expand_env_var(password, &secrets).unwrap(), "s3cret");
                    }
                    _ => panic!("Expected Basic auth"),
                }
            }
            StoreDef::Local { .. } => panic!("Expected Caldav store"),
        }

        // Calendar href
        assert_eq!(
            expand_env_var(
                config.calendars[0].calendar_href.as_deref().unwrap(),
                &secrets
            )
            .unwrap(),
            "/dav/admin/work/"
        );
    }

    #[test]
    fn secrets_files_parsed_from_toml() {
        const TOML: &str = r#"
secrets_files = [".env", "/etc/aim/secrets"]

[stores.local]
type = "local"

[[calendars]]
id = "default"
name = "Default"
store = "local"
"#;

        let config: Config = toml::from_str(TOML).expect("Failed to parse TOML");
        assert_eq!(config.secrets_files, vec![".env", "/etc/aim/secrets"]);
    }
}
