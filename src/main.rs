use std::{io::Error, net::TcpListener};

use zero2prod::startup;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let listener = TcpListener::bind("127.0.0.1:8000")?;
    startup::run(listener)?.await
}
