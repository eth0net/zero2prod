use config::{Config, ConfigError, File, FileFormat};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub application_port: u16,
    pub database: DatabaseSettings,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!("{}/{}", self.connection_string_no_db(), self.database)
    }

    pub fn connection_string_no_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}

pub fn get_config() -> Result<Settings, ConfigError> {
    let settings = Config::builder()
        .add_source(File::new("config.yaml", FileFormat::Yaml))
        .build()?;
    settings.try_deserialize::<Settings>()
}
