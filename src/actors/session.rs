use actix::fut;
use actix::prelude::*;
use actix_web_actors::ws;

use crate::actors::syncserver::SyncServer;
use crate::message::clientmessage::*;

#[derive(Default)]
pub struct SyncClientSession {
    username: Option<String>,
}

impl SyncClientSession {
    pub fn initiate_sync(&mut self, username: String, ctx: &mut ws::WebsocketContext<Self>) {
        SyncServer::from_registry()
            .send(InitiateSync(ctx.address().recipient(), username))
            .into_actor(self)
            .then(|_, _, _| fut::ready(()))
            .wait(ctx);
    }
}

impl Handler<ScheduleMessage> for SyncClientSession {
    type Result = ();
    fn handle(&mut self, msg: ScheduleMessage, ctx: &mut Self::Context) {}
}

impl Actor for SyncClientSession {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for SyncClientSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        match msg {
            ws::Message::Text(text) => {
                if text == "test" {
                    self.initiate_sync("ying".to_string(), ctx);
                }
            }
            _ => {}
        }
    }
}
