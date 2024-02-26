use std::io::Result;
use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::config::{database, Config};
use crate::mail;
use crate::routes::{health, subscribe};

pub struct Application {
    pub port: u16,
    pub server: Server,
}

impl Application {
    pub async fn build(config: Config) -> std::io::Result<Self> {
        let db_pool = get_db_pool(&config.database);
        let mail_client = mail::Client::new(config.mail).expect("get mail client");
        let address = format!("{}:{}", config.application.host, config.application.port);
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr()?.port();
        let server = run(listener, db_pool, mail_client)?;
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<()> {
        self.server.await
    }
}

pub fn get_db_pool(config: &database::Config) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(config.with_db())
}

fn run(listener: TcpListener, db_pool: PgPool, mail_client: mail::Client) -> Result<Server> {
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health", web::get().to(health))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone())
            .app_data(mail_client.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
