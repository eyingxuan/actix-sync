use actix::fut;
use actix::prelude::*;
use actix_web_actors::ws;
use crdts::orswot::Orswot;
use crdts::CmRDT;

use crate::actors::syncserver::SyncServer;
use crate::message::clientmessage::*;

enum CourseUpdate {
    Add(String),
    Rem(String),
}

#[derive(Default)]
pub struct SyncClientSession {
    username: Option<String>,
    schedule_crdt: Option<Orswot<String, u8>>,
    id: u8,
}

impl SyncClientSession {
    fn initiate_sync(&mut self, username: String, ctx: &mut ws::WebsocketContext<Self>) {
        SyncServer::from_registry()
            .send(InitiateSync(ctx.address().recipient(), username.clone()))
            .into_actor(self)
            .then(|res, act, ctx| {
                let opt = res
                    .expect("server actor must be active")
                    .expect("infallible error");
                match opt {
                    None => ctx.text("invalid username"),
                    Some((crdt, id)) => {
                        act.schedule_crdt = Some(crdt);
                        act.id = id;
                        act.username = Some(username);
                        ctx.text("initiation success");
                    }
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn update_course(&mut self, course_upd: CourseUpdate, ctx: &mut ws::WebsocketContext<Self>) {
        let crdt = self.schedule_crdt.as_mut().unwrap();
        let crdt_ctx = crdt.read_ctx();
        let update = match course_upd {
            CourseUpdate::Add(c) => crdt.add(c, crdt_ctx.derive_add_ctx(self.id)),
            CourseUpdate::Rem(c) => crdt.rm(c, crdt_ctx.derive_rm_ctx()),
        };

        crdt.apply(update.clone());
        ctx.text(format!("{:?}", crdt.read().val));
        SyncServer::from_registry()
            .send(UpdateSchedule(
                self.username.clone().expect("must initiate sync"),
                ctx.address().recipient(),
                update,
            ))
            .into_actor(self)
            .then(|_, _, _| fut::ready(()))
            .spawn(ctx);
    }
}

impl Handler<ScheduleMessage> for SyncClientSession {
    type Result = ();
    fn handle(&mut self, msg: ScheduleMessage, ctx: &mut Self::Context) {
        let ScheduleMessage(op) = msg;
        let crdt = self.schedule_crdt.as_mut().unwrap();
        crdt.apply(op);
        ctx.text(format!("{:?}", crdt.read().val));
    }
}

impl Actor for SyncClientSession {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for SyncClientSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // TODO: Heartbeats and handle disconnection
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        match msg {
            ws::Message::Text(text) => {
                let cmds = text.split(" ").collect::<Vec<&str>>();
                if cmds[0] == "/join" {
                    self.initiate_sync(cmds[1].to_owned(), ctx);
                } else if cmds[0] == "/add" {
                    self.update_course(CourseUpdate::Add(cmds[1].to_owned()), ctx);
                } else if cmds[0] == "/rem" {
                    self.update_course(CourseUpdate::Rem(cmds[1].to_owned()), ctx);
                }
            }
            _ => {}
        }
    }
}
