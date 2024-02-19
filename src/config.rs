use config::{Config, ConfigError, File, FileFormat};
use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub application_port: u16,
    pub database: DatabaseSettings,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Secret<String>,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "{}/{}",
            self.connection_string_no_db().expose_secret(),
            self.name
        ))
    }

    pub fn connection_string_no_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}

pub fn get_config() -> Result<Settings, ConfigError> {
    let settings = Config::builder()
        .add_source(File::new("config.yaml", FileFormat::Yaml))
        .build()?;
    settings.try_deserialize::<Settings>()
}
