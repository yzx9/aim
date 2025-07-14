use aim_core::Config;
use std::{error::Error, path::PathBuf};
use xdg::BaseDirectories;

pub const APP_NAME: &str = "aim";

pub async fn parse_config(path: Option<&PathBuf>) -> Result<Config, Box<dyn Error>> {
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
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let table: toml::Table = content
        .parse()
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    let calendar_path = table
        .get("calendar_path")
        .and_then(|v| v.as_str())
        .ok_or("Missing or invalid 'calendar_path' in config")?;

    let calendar_path = expand_path(calendar_path)?;

    Ok(Config::new(calendar_path))
}

fn expand_path(path: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Handle tilde (~) and environment variables in the path
    for prefix in ["~/", "$HOME/", "${HOME}/"] {
        if path.starts_with(prefix) {
            let home = home::home_dir().ok_or("User-specific home directory not found")?;
            return Ok(home.join(&path[prefix.len()..]));
        }
    }

    // Handle XDG_CONFIG_HOME
    for prefix in ["$XDG_CONFIG_HOME/", "${XDG_CONFIG_HOME}/"] {
        if path.starts_with(prefix) {
            let config_home = BaseDirectories::with_prefix(APP_NAME)
                .get_config_home()
                .ok_or("User-specific config directory not found")?;

            let rest = &path[prefix.len()..];
            let app_name = APP_NAME.to_string() + "/";
            if rest.starts_with(&app_name) {
                // If the path starts with app name, we assume it's relative to the config home
                return Ok(config_home.join(&rest[APP_NAME.len() + 1..]));
            }

            return Ok(config_home.join(rest));
        }
    }

    Ok(path.into())
}
