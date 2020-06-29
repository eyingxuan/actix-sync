use actix::prelude::*;
use actix_web_actors::ws;

#[derive(Default)]
pub struct SyncClientSession {
    username: Option<String>,
}

impl Actor for SyncClientSession {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for SyncClientSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {}
}
