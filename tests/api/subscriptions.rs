use reqwest::StatusCode;

use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_ok() {
    let app = spawn_app().await;

    let body = "name=Totally%20Real%20Name&email=trn%40mail.tld";

    let response = app.post_subscriptions(body.into()).await;

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

    let test_cases = vec![
        ("name=Totally%20Real%20Name", "missing the email"),
        ("email=trn%mail.tld", "missing the name"),
        ("", "missing both name and email"),
        ("name=Totally%20Real%20Name&email=", "empty email"),
        ("name=&email=trn%40mail.tld", "empty name"),
        ("name=&email=", "empty name and email"),
    ];

    for (body, description) in test_cases {
        let response = app.post_subscriptions(body.into()).await;

        assert_eq!(
            StatusCode::BAD_REQUEST,
            response.status(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            description
        );
    }
}
