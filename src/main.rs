use actix::registry::SystemService;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use actors::{dbserver, session, syncserver};
use dotenv::dotenv;

mod actors;
mod message;

async fn sync_route(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(session::SyncClientSession::default(), &req, stream)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let addr = "127.0.0.1:8000";
    syncserver::SyncServer::from_registry();
    dbserver::DbServer::from_registry();

    let server = HttpServer::new(move || App::new().service(web::resource("/ws/").to(sync_route)))
        .bind(&addr)?;

    server.run().await
}
