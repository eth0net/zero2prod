use std::{io::Error, net::TcpListener};

use actix_web::{dev::Server, web::get, App, HttpResponse, HttpServer};

async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener) -> Result<Server, Error> {
    let server = HttpServer::new(|| App::new().route("/health", get().to(health)))
        .listen(listener)?
        .run();
    Ok(server)
}
