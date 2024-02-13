use std::net::TcpListener;

use reqwest::StatusCode;
use sqlx::{Connection, PgConnection};
use zero2prod::config::get_config;

#[tokio::test]
async fn health() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_ok() {
    let address = spawn_app();
    let config = get_config().expect("Failed to read configuration.");
    let connection_string = config.database.connection_string();
    let connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");
    let client = reqwest::Client::new();

    let body = "name=Totally%20Real%20Name&email=trn%mail.tld";

    let response = client
        .post(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(StatusCode::OK, response.status())
}

#[tokio::test]
async fn subscribe_bad_request() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=Totally%20Real%20Name", "missing the email"),
        ("email=trn%mail.tld", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (body, description) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &address))
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

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind address");
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::startup::run(listener).expect("Failed to run server");

    tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
