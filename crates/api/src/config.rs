use std::env;

use thiserror::Error;
use url::Url;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub app_url: Url,
    pub api_url: Url,
    pub auth: Option<AuthConfig>,
    pub session_cookie_name: String,
    pub session_cookie_secure: bool,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub google_client_id: String,
    pub google_client_secret: String,
    pub session_secret: String,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("{name} must be a valid URL")]
    InvalidUrl {
        name: &'static str,
        #[source]
        source: url::ParseError,
    },
    #[error("production runtime configuration is invalid: {0}")]
    ProductionValidation(String),
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let app_url = match env_url("APP_URL")? {
            Some(url) => url,
            None => env_url("PUBLIC_APP_URL")?.unwrap_or_else(|| {
                Url::parse("http://localhost:3015").expect("valid default app URL")
            }),
        };
        let api_url = env_url("API_URL")?
            .unwrap_or_else(|| Url::parse("http://localhost:3016").expect("valid default API URL"));
        let session_cookie_name =
            env::var("SESSION_COOKIE_NAME").unwrap_or_else(|_| "__Host-session".to_owned());
        let session_cookie_secure = env::var("SESSION_COOKIE_SECURE")
            .map(|value| parse_boolish(&value))
            .unwrap_or_else(|_| is_deployed_env() || !is_local_url(&api_url));

        let config = Self {
            app_url,
            api_url,
            auth: AuthConfig::from_env(),
            session_cookie_name,
            session_cookie_secure,
        };

        config.validate_production()?;

        Ok(config)
    }

    pub fn local_development() -> Self {
        Self {
            app_url: Url::parse("http://localhost:3015").expect("valid local app URL"),
            api_url: Url::parse("http://localhost:3016").expect("valid local API URL"),
            auth: None,
            session_cookie_name: "__Host-session".to_owned(),
            session_cookie_secure: false,
        }
    }

    fn validate_production(&self) -> Result<(), ConfigError> {
        if !is_deployed_env() {
            return Ok(());
        }

        let mut errors = Vec::new();
        for name in [
            "APP_URL",
            "PUBLIC_APP_URL",
            "API_URL",
            "SESSION_SECRET",
            "AUTH_GOOGLE_ID",
            "AUTH_GOOGLE_SECRET",
            "DATABASE_URL",
        ] {
            if non_empty_env(name).is_none() {
                errors.push(format!("{name} is required"));
            }
        }

        if !self.session_cookie_secure {
            errors.push("SESSION_COOKIE_SECURE must be true in staging/production".to_owned());
        }
        if !uses_https(&self.app_url) {
            errors.push("APP_URL/PUBLIC_APP_URL must use https in staging/production".to_owned());
        }
        if !uses_https(&self.api_url) {
            errors.push("API_URL must use https in staging/production".to_owned());
        }
        if self.auth.is_none() {
            errors.push(
                "AUTH_GOOGLE_ID, AUTH_GOOGLE_SECRET, and SESSION_SECRET must all be configured"
                    .to_owned(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ConfigError::ProductionValidation(errors.join("; ")))
        }
    }
}

impl AuthConfig {
    fn from_env() -> Option<Self> {
        let google_client_id = non_empty_env("AUTH_GOOGLE_ID")?;
        let google_client_secret = non_empty_env("AUTH_GOOGLE_SECRET")?;
        let session_secret = non_empty_env("SESSION_SECRET")?;

        Some(Self {
            google_client_id,
            google_client_secret,
            session_secret,
        })
    }
}

fn env_url(name: &'static str) -> Result<Option<Url>, ConfigError> {
    let Some(value) = non_empty_env(name) else {
        return Ok(None);
    };
    Url::parse(&value)
        .map(Some)
        .map_err(|source| ConfigError::InvalidUrl { name, source })
}

fn non_empty_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn is_local_url(url: &Url) -> bool {
    matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1"))
}

fn uses_https(url: &Url) -> bool {
    url.scheme() == "https"
}

fn parse_boolish(value: &str) -> bool {
    !matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "false" | "0" | "no"
    )
}

pub fn is_deployed_env() -> bool {
    ["APP_ENV", "RAILS_ENV", "ENVIRONMENT", "NODE_ENV"]
        .iter()
        .filter_map(|name| env::var(name).ok())
        .any(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "production" | "prod" | "staging"
            )
        })
}

pub fn api_port_from_env() -> Result<u16, ConfigError> {
    match non_empty_env("PORT") {
        Some(value) => value.parse::<u16>().map_err(|_| {
            ConfigError::ProductionValidation(
                "PORT must be an integer between 0 and 65535".to_owned(),
            )
        }),
        None => Ok(3016),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn with_clean_env(test: impl FnOnce()) {
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let names = [
            "APP_ENV",
            "RAILS_ENV",
            "ENVIRONMENT",
            "NODE_ENV",
            "APP_URL",
            "PUBLIC_APP_URL",
            "API_URL",
            "SESSION_COOKIE_SECURE",
            "SESSION_SECRET",
            "AUTH_GOOGLE_ID",
            "AUTH_GOOGLE_SECRET",
            "DATABASE_URL",
            "PORT",
        ];
        let original: Vec<_> = names
            .iter()
            .map(|name| (*name, env::var(name).ok()))
            .collect();
        for name in names {
            env::remove_var(name);
        }
        test();
        for (name, value) in original {
            match value {
                Some(value) => env::set_var(name, value),
                None => env::remove_var(name),
            }
        }
    }

    #[test]
    fn api_port_defaults_to_3016() {
        with_clean_env(|| assert_eq!(api_port_from_env().unwrap(), 3016));
    }

    #[test]
    fn api_port_respects_port_env() {
        with_clean_env(|| {
            env::set_var("PORT", "4017");
            assert_eq!(api_port_from_env().unwrap(), 4017);
        });
    }

    #[test]
    fn production_requires_secure_cookie_and_required_envs() {
        with_clean_env(|| {
            env::set_var("APP_ENV", "production");
            env::set_var("APP_URL", "https://app.example.com");
            env::set_var("PUBLIC_APP_URL", "https://app.example.com");
            env::set_var("API_URL", "https://api.example.com");
            env::set_var("SESSION_COOKIE_SECURE", "false");

            let error = AppConfig::from_env().unwrap_err().to_string();

            assert!(error.contains("SESSION_SECRET is required"));
            assert!(error.contains("AUTH_GOOGLE_ID is required"));
            assert!(error.contains("AUTH_GOOGLE_SECRET is required"));
            assert!(error.contains("DATABASE_URL is required"));
            assert!(error.contains("SESSION_COOKIE_SECURE must be true"));
        });
    }

    #[test]
    fn staging_defaults_session_cookie_secure_to_true() {
        with_clean_env(|| {
            env::set_var("APP_ENV", "staging");
            env::set_var("APP_URL", "https://app.example.com");
            env::set_var("PUBLIC_APP_URL", "https://app.example.com");
            env::set_var("API_URL", "https://api.example.com");
            env::set_var("SESSION_SECRET", "secret");
            env::set_var("AUTH_GOOGLE_ID", "google-id");
            env::set_var("AUTH_GOOGLE_SECRET", "google-secret");
            env::set_var("DATABASE_URL", "postgresql://example");

            let config = AppConfig::from_env().unwrap();

            assert!(config.session_cookie_secure);
        });
    }
}
