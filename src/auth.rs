use crate::config::AppConfig;
use std::sync::Arc;

pub struct Authenticator {
    expected_key: Option<String>,
}

impl Authenticator {
    pub fn new(cfg: &Arc<AppConfig>) -> Self {
        let expected_key = cfg
            .auth
            .api_key_env
            .as_ref()
            .and_then(|var| std::env::var(var).ok());

        if expected_key.is_some() {
            tracing::info!("API key authentication enabled");
        }

        Self { expected_key }
    }

    pub fn validate(&self, header: Option<&str>) -> bool {
        let Some(key) = &self.expected_key else {
            return true;
        };
        header.map_or(false, |h| {
            h.strip_prefix("Bearer ")
                .map_or(false, |token| token == key)
        })
    }

    pub fn is_enabled(&self) -> bool {
        self.expected_key.is_some()
    }
}
