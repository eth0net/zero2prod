use std::{io::Error, net::TcpListener};

use sqlx::PgPool;
use zero2prod::{config::get_config, startup};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = get_config().expect("Failed to read configuration.");

    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(address)?;

    let connection_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    startup::run(listener, connection_pool)?.await
}
