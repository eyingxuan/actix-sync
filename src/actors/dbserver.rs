use crate::message::dbmessage::*;
use actix::prelude::*;
use models::*;
use mongodb::bson::{doc, from_bson, Bson};
use mongodb::error::Error;
use mongodb::Client;
use std::env::var;

pub mod models;

#[derive(Default)]
pub struct DbServer {
    handle: Option<Client>,
}

impl Actor for DbServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let client = async {
            let mongo_uri = var("MONGO_URI").expect("MONGO_URI must be present in .env file");
            let handle = Client::with_uri_str(mongo_uri.as_str())
                .await
                .expect("db connection must not fail");
            handle
        };

        ctx.wait(client.into_actor(self).then(|res, act, _| {
            act.handle = Some(res);
            actix::fut::ready(())
        }));
    }
}

impl Handler<DbGetSchedule> for DbServer {
    type Result = ResponseFuture<Result<Option<Schedule>, Error>>;
    fn handle(&mut self, msg: DbGetSchedule, ctx: &mut Self::Context) -> Self::Result {
        let DbGetSchedule(user) = msg;
        let conn = self.handle.clone().expect("handle must exist");
        Box::pin(async move {
            let col = conn.database("sync").collection("sched");
            let doc = col.find_one(Some(doc! { "username": user }), None).await?;
            if doc.is_none() {
                return Ok(None);
            }

            // unwrap is ok because check is done above
            let schedule = from_bson(Bson::from(doc.unwrap()))?;
            Ok(Some(schedule))
        })
    }
}

impl SystemService for DbServer {}
impl Supervised for DbServer {}
