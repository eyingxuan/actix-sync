use actix::fut;
use actix::prelude::*;
use actix_web_actors::ws;
use crdts::orswot::Orswot;
use crdts::CmRDT;

use crate::actors::syncserver::SyncServer;
use crate::message::clientmessage::*;
use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(5);

enum CourseUpdate {
    Add(String),
    Rem(String),
}

pub struct SyncClientSession {
    username: Option<String>,
    schedule_crdt: Option<Orswot<String, u8>>,
    id: u8,
    hb: Instant,
}

impl Default for SyncClientSession {
    fn default() -> Self {
        SyncClientSession {
            username: None,
            schedule_crdt: None,
            id: 0,
            hb: Instant::now(),
        }
    }
}

impl SyncClientSession {
    fn format_schedule(val: HashSet<String>) -> String {
        format!("{:?}", val.into_iter().collect::<Vec<String>>())
    }

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
                        let crdt_val = crdt.read().val;
                        act.schedule_crdt = Some(crdt);
                        act.id = id;
                        act.username = Some(username);
                        ctx.text(SyncClientSession::format_schedule(crdt_val));
                        // ctx.text("initiation success");
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
        ctx.text(SyncClientSession::format_schedule(crdt.read().val));
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

    fn create_user(&mut self, username: String, ctx: &mut ws::WebsocketContext<Self>) {
        let msg = CreateUser(username);
        SyncServer::from_registry()
            .send(msg)
            .into_actor(self)
            .then(|res, _, _| {
                if res
                    .expect("assume channel did not fail")
                    .expect("infallible")
                {
                    // ctx.text("succeeded");
                } else {
                    // ctx.text("failed");
                }
                fut::ready(())
            })
            .spawn(ctx);
    }

    fn disconnect_session(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        // TODO: fix unwrap
        let msg = DisconnectSync(
            self.username.clone().unwrap(),
            self.id,
            ctx.address().recipient(),
        );
        SyncServer::from_registry()
            .send(msg)
            .into_actor(self)
            .then(|_, _, _| fut::ready(()))
            .wait(ctx);
    }
}

impl Handler<ScheduleMessage> for SyncClientSession {
    type Result = ();
    fn handle(&mut self, msg: ScheduleMessage, ctx: &mut Self::Context) {
        let ScheduleMessage(op) = msg;
        let crdt = self.schedule_crdt.as_mut().unwrap();
        crdt.apply(op);
        ctx.text(SyncClientSession::format_schedule(crdt.read().val));
    }
}

impl Actor for SyncClientSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                act.disconnect_session(ctx);
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
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
                } else if cmds[0] == "/create" {
                    self.create_user(cmds[1].to_owned(), ctx);
                }
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            _ => {}
        }
    }
}
