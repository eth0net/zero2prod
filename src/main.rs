use std::io::{stdout, Result};
use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::PgPool;
use zero2prod::config::get_config;
use zero2prod::startup::run;
use zero2prod::telemetry::{init_subscriber, make_subscriber};

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = make_subscriber("zero2prod".into(), "info".into(), stdout);
    init_subscriber(subscriber);

    let config = get_config().expect("Failed to read configuration.");

    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(address)?;

    let connection_pool = PgPool::connect(config.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");

    run(listener, connection_pool)?.await
}
