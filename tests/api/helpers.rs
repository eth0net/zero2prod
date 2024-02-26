use std::env::var;
use std::io::{sink, stdout};

use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::config::{database, get_config};
use zero2prod::startup::{get_db_pool, Application};
use zero2prod::telemetry::{init_subscriber, make_subscriber};

static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber_name = "test".to_string();
    let default_level = "info".to_string();
    match var("TEST_LOG") {
        Ok(_) => {
            let subscriber = make_subscriber(subscriber_name, default_level, stdout);
            init_subscriber(subscriber);
        }
        Err(_) => {
            let subscriber = make_subscriber(subscriber_name, default_level, sink);
            init_subscriber(subscriber);
        }
    };
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let config = {
        let mut config = get_config().expect("Failed to read config.");
        config.database.name = Uuid::new_v4().to_string();
        config.application.port = 0;
        config
    };

    configure_database(&config.database).await;

    let application = Application::build(config.clone())
        .await
        .expect("Failed to build app.");
    let address = format!("http://localhost:{}", application.port());
    tokio::spawn(application.run_until_stopped());

    TestApp {
        address,
        db_pool: get_db_pool(&config.database),
    }
}

async fn configure_database(config: &database::Config) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres.");

    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.name))
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database.");

    connection_pool
}
