use crate::dbserver::DbServer;
use crate::message::clientmessage::*;
use crate::message::dbmessage::*;
use actix::prelude::*;
use std::collections::HashMap;

type ClientRecv = Recipient<ScheduleMessage>;

#[derive(Default)]
pub struct SyncServer {
    observers: HashMap<String, Vec<ClientRecv>>,
}

impl Handler<InitiateSync> for SyncServer {
    type Result = ResponseFuture<Result<bool, std::convert::Infallible>>;

    fn handle(&mut self, msg: InitiateSync, _ctx: &mut Self::Context) -> Self::Result {
        let InitiateSync(recp, username) = msg;

        Box::pin(async move {
            let res = DbServer::from_registry()
                .send(DbGetSchedule(username))
                .await
                .unwrap()
                .unwrap();

            if res.is_some() {
                println!("{:?}", res.unwrap());
                return Ok(true);
            } else {
                return Ok(false);
            }
        })
    }
}

impl Actor for SyncServer {
    type Context = Context<Self>;
}

impl SystemService for SyncServer {}
impl Supervised for SyncServer {}
