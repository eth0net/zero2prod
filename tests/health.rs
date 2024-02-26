use std::env::var;
use std::io::{sink, stdout};
use std::net::TcpListener;

use once_cell::sync::Lazy;
use reqwest::StatusCode;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::config::{database, get_config};
use zero2prod::startup::run;
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

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let mut config = get_config().expect("Failed to read config.");
    config.database.name = Uuid::new_v4().to_string();

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind address");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let db_pool = configure_database(&config.database).await;

    let server = run(listener, db_pool.clone()).expect("Failed to run server");
    tokio::spawn(server);

    TestApp { address, db_pool }
}

pub async fn configure_database(config: &database::Config) -> PgPool {
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

#[tokio::test]
async fn health() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_ok() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=Totally%20Real%20Name&email=trn%40mail.tld";

    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(StatusCode::OK, response.status());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "trn@mail.tld");
    assert_eq!(saved.name, "Totally Real Name")
}

#[tokio::test]
async fn subscribe_bad_request() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=Totally%20Real%20Name", "missing the email"),
        ("email=trn%mail.tld", "missing the name"),
        ("", "missing both name and email"),
        ("name=Totally%20Real%20Name&email=", "empty email"),
        ("name=&email=trn%40mail.tld", "empty name"),
        ("name=&email=", "empty name and email"),
    ];

    for (body, description) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            StatusCode::BAD_REQUEST,
            response.status(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            description
        );
    }
}
