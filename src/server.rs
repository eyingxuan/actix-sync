use crate::message::*;
use actix::prelude::*;
use actix_broker::BrokerSubscribe;
use mongodb::bson::doc;
use mongodb::Client;
use std::collections::HashMap;

type ClientRecv = Recipient<ScheduleMessage>;

#[derive(Default)]
pub struct SyncServer {
    handle: Option<Client>,
    observers: HashMap<String, Vec<ClientRecv>>,
}

impl Handler<InitiateSync> for SyncServer {
    type Result = ResponseFuture<Result<bool, std::convert::Infallible>>;

    fn handle(&mut self, msg: InitiateSync, _ctx: &mut Self::Context) -> Self::Result {
        let InitiateSync(recp, username) = msg;
        let conn = self.handle.clone().expect("connection must be available");
        Box::pin(async move {
            let col = conn.database("sync").collection("sched");
            Ok(col
                .find_one(Some(doc! {"username": username}), None)
                .await
                .unwrap()
                .is_some())
        })
    }
}

impl Actor for SyncServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // TODO: Subscriptions
        let client = async { Client::with_uri_str("mongodb://example.com").await.unwrap() };
        ctx.wait(client.into_actor(self).then(|res, act, _ctx| {
            act.handle = Some(res);
            actix::fut::ready(())
        }));
    }
}

impl SystemService for SyncServer {}
impl Supervised for SyncServer {}
