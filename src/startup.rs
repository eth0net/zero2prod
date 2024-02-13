use std::{io::Error, net::TcpListener};

use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, Error> {
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .route("/health", web::get().to(crate::routes::health))
            .route("/subscriptions", web::post().to(crate::routes::subscribe))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
