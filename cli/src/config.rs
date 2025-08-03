// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;

use aimcal_core::APP_NAME;
use aimcal_core::Config as CoreConfig;

pub async fn parse_config(path: Option<PathBuf>) -> Result<(CoreConfig, Config), Box<dyn Error>> {
    let path = match path {
        Some(path) => path,
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

    let raw: ConfigRaw = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read config file: {e}"))?
        .parse()?;

    Ok((raw.0, Config {}))
}

/// Configuration for the Aim application.
#[derive(Debug, Clone, Copy, serde::Deserialize)]
pub struct Config;

#[derive(Debug, serde::Deserialize)]
struct ConfigRaw(CoreConfig);

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
    config_dir.ok_or("User-specific home directory not found".into())
}
