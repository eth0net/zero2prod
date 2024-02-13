use std::{io::Error, net::TcpListener};

use actix_web::{dev::Server, web, App, HttpServer};

pub fn run(listener: TcpListener) -> Result<Server, Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(crate::routes::health))
            .route("/subscriptions", web::post().to(crate::routes::subscribe))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
