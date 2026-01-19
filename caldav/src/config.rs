// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

/// `CalDAV` authentication method.
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(tag = "type")]
pub enum AuthMethod {
    /// No authentication.
    #[serde(rename = "none")]
    #[default]
    None,
    /// Basic authentication (username/password).
    #[serde(rename = "basic")]
    Basic {
        /// Username for authentication.
        username: String,
        /// Password for authentication.
        password: String,
    },
    /// Bearer token authentication (OAuth).
    #[serde(rename = "bearer")]
    Bearer {
        /// Bearer token.
        token: String,
    },
}

/// `CalDAV` server configuration.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CalDavConfig {
    /// Base URL of the `CalDAV` server.
    pub base_url: String,
    /// Calendar home path (e.g., /dav/calendars/user/).
    pub calendar_home: String,
    /// Authentication method.
    #[serde(default)]
    pub auth: AuthMethod,
    /// Request timeout in seconds.
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// User agent string.
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
}

const fn default_timeout() -> u64 {
    30
}

fn default_user_agent() -> String {
    concat!("aimcal-caldav/", env!("CARGO_PKG_VERSION")).to_string()
}

impl Default for CalDavConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            calendar_home: String::new(),
            auth: AuthMethod::default(),
            timeout_secs: default_timeout(),
            user_agent: default_user_agent(),
        }
    }
}
