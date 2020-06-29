use actix_web::{App, HttpServer};

mod db;
mod message;
mod server;
mod session;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:8000";

    let server = HttpServer::new(move || {
        App::new()
        // .service(web::resource("/ws/").to())
    })
    .bind(&addr)?;

    server.run().await
}
