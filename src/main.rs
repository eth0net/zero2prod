use std::io::{stdout, Result};

use zero2prod::config::get_config;
use zero2prod::startup::Application;
use zero2prod::telemetry::{init_subscriber, make_subscriber};

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = make_subscriber("zero2prod".into(), "info".into(), stdout);
    init_subscriber(subscriber);

    let config = get_config().expect("Failed to read configuration.");
    let application = Application::build(config).await?;
    application.run_until_stopped().await?;
    Ok(())
}
