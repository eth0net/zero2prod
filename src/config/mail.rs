use std::time::Duration;

use secrecy::Secret;

use crate::domain::SubscriberEmail;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub auth_token: Secret<String>,
    pub base_url: String,
    pub sender: String,
    pub timeout: Duration,
}

impl Config {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender.clone())
    }
}
