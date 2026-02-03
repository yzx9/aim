// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{error::Error, path::PathBuf, str::FromStr};

use tokio::fs;

use aimcal_core::{APP_NAME, Config as CoreConfig};

const AIM_CONFIG_ENV: &str = "AIM_CONFIG";

#[tracing::instrument]
pub async fn parse_config(path: Option<PathBuf>) -> Result<(CoreConfig, Config), Box<dyn Error>> {
    let path = if let Some(path) = path {
        path
    } else if let Ok(env_path) = std::env::var(AIM_CONFIG_ENV) {
        PathBuf::from(env_path)
    } else {
        // TODO: search config in multiple locations
        let config = get_config_dir()?.join(format!("{APP_NAME}/config.toml"));
        if !config.exists() {
            return Err(format!("No config found at: {}", config.display()).into());
        }
        config
    };

    fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Failed to read config file at {}: {}", path.display(), e))?
        .parse::<ConfigRaw>()
        .map(|a| (a.core, Config {}))
}

/// Configuration for the Aim application.
#[derive(Debug, Clone, Copy, serde::Deserialize)]
pub struct Config;

#[derive(Debug, serde::Deserialize)]
struct ConfigRaw {
    core: CoreConfig,
}

impl FromStr for ConfigRaw {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(toml::from_str(s)?)
    }
}

fn get_config_dir() -> Result<PathBuf, Box<dyn Error>> {
    #[cfg(unix)]
    let config_dir = xdg::BaseDirectories::new().get_config_home();
    #[cfg(windows)]
    let config_dir = dirs::config_dir();
    config_dir.ok_or_else(|| "User-specific home directory not found".into())
}

#[cfg(test)]
#[allow(unsafe_code)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn cli_flag_overrides_env_var() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let calendar_dir = temp_dir.path().join("calendar");
        fs::create_dir(&calendar_dir).unwrap();

        let toml_content = format!(
            r#"
[core]
calendar_path = "{}"
"#,
            calendar_dir.display()
        );
        fs::write(&config_path, toml_content).unwrap();

        let env_path = temp_dir.path().join("env_config.toml");
        let env_calendar_dir = temp_dir.path().join("env_calendar");
        fs::create_dir(&env_calendar_dir).unwrap();
        let env_toml_content = format!(
            r#"
[core]
calendar_path = "{}"
"#,
            env_calendar_dir.display()
        );
        fs::write(&env_path, env_toml_content).unwrap();

        unsafe {
            std::env::set_var(AIM_CONFIG_ENV, &env_path);
        }

        let (config, _) = parse_config(Some(config_path.clone())).await.unwrap();

        assert_eq!(config.calendar_path, calendar_dir);

        unsafe {
            std::env::remove_var(AIM_CONFIG_ENV);
        }
    }

    #[tokio::test]
    async fn env_var_overrides_default_config() {
        let temp_dir = TempDir::new().unwrap();
        let env_config_path = temp_dir.path().join("env_config.toml");
        let calendar_dir = temp_dir.path().join("calendar");
        fs::create_dir(&calendar_dir).unwrap();

        let toml_content = format!(
            r#"
[core]
calendar_path = "{}"
"#,
            calendar_dir.display()
        );
        fs::write(&env_config_path, toml_content).unwrap();

        unsafe {
            std::env::set_var(AIM_CONFIG_ENV, &env_config_path);
        }

        let (config, _) = parse_config(None).await.unwrap();

        assert_eq!(config.calendar_path, calendar_dir);

        unsafe {
            std::env::remove_var(AIM_CONFIG_ENV);
        }
    }

    #[tokio::test]
    async fn respects_priority_order() {
        let temp_dir = TempDir::new().unwrap();

        let cli_config_path = temp_dir.path().join("cli_config.toml");
        let cli_calendar_dir = temp_dir.path().join("cli_calendar");
        fs::create_dir(&cli_calendar_dir).unwrap();
        let cli_toml_content = format!(
            r#"
[core]
calendar_path = "{}"
"#,
            cli_calendar_dir.display()
        );
        fs::write(&cli_config_path, cli_toml_content).unwrap();

        let env_config_path = temp_dir.path().join("env_config.toml");
        let env_calendar_dir = temp_dir.path().join("env_calendar");
        fs::create_dir(&env_calendar_dir).unwrap();
        let env_toml_content = format!(
            r#"
[core]
calendar_path = "{}"
"#,
            env_calendar_dir.display()
        );
        fs::write(&env_config_path, env_toml_content).unwrap();

        unsafe {
            std::env::set_var(AIM_CONFIG_ENV, &env_config_path);
        }

        let (config, _) = parse_config(Some(cli_config_path)).await.unwrap();

        assert_eq!(config.calendar_path, cli_calendar_dir);

        unsafe {
            std::env::remove_var(AIM_CONFIG_ENV);
        }
    }

    #[tokio::test]
    async fn uses_default_when_no_cli_or_env() {
        let temp_dir = TempDir::new().unwrap();
        let default_config_dir = temp_dir.path().join("aim");
        fs::create_dir_all(&default_config_dir).unwrap();
        let default_config_path = default_config_dir.join("config.toml");
        let calendar_dir = temp_dir.path().join("calendar");
        fs::create_dir(&calendar_dir).unwrap();

        let toml_content = format!(
            r#"
[core]
calendar_path = "{}"
"#,
            calendar_dir.display()
        );
        fs::write(&default_config_path, toml_content).unwrap();

        unsafe {
            std::env::remove_var(AIM_CONFIG_ENV);
        }
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        }

        let (config, _) = parse_config(None).await.unwrap();

        assert_eq!(config.calendar_path, calendar_dir);

        unsafe {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
    }

    #[tokio::test]
    async fn returns_error_when_no_config_found() {
        let temp_dir = TempDir::new().unwrap();
        let empty_dir = temp_dir.path().join("empty");
        fs::create_dir(&empty_dir).unwrap();

        unsafe {
            std::env::remove_var(AIM_CONFIG_ENV);
        }
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", &empty_dir);
        }

        let result = parse_config(None).await;

        assert!(result.is_err());

        unsafe {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
    }
}
