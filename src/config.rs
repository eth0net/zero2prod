use std::env;

use config::File;

use self::environment::Environment;

pub mod application;
pub mod database;
pub mod environment;
pub mod mail;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub application: application::Config,
    pub database: database::Config,
    pub mail: mail::Config,
}

pub fn get_config() -> Result<Config, config::ConfigError> {
    let base_path = env::current_dir().expect("Failed to determine the current directory.");
    let config_dir = base_path.join("config");

    let environment: Environment = env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse environment");

    let config_file_path = config_dir.join("config.yaml");
    let environment_file_path = config_dir.join(format!("{}.yaml", environment.as_str()));

    let config_file = File::from(config_file_path);
    let environment_file = File::from(environment_file_path);
    let environment_vars = config::Environment::with_prefix("app")
        .prefix_separator("_")
        .separator("__");

    let settings = config::Config::builder()
        .add_source(config_file)
        .add_source(environment_file)
        .add_source(environment_vars)
        .build()?;
    settings.try_deserialize::<Config>()
}
