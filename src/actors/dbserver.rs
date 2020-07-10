use crate::message::dbmessage::*;
use actix::prelude::*;
use models::*;
use mongodb::bson::{doc, from_bson, to_bson, Bson};
use mongodb::error::Error;
use mongodb::options::{UpdateModifications, UpdateOptions};
use mongodb::Client;
use std::env::var;

pub mod models;

const DATABASE: &'static str = "sync";
const SCHED_COL: &'static str = "sched";
const COURSE_COL: &'static str = "courses";

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
    fn handle(&mut self, msg: DbGetSchedule, _ctx: &mut Self::Context) -> Self::Result {
        let DbGetSchedule(user) = msg;
        let conn = self.handle.clone().expect("handle must exist");
        Box::pin(async move {
            let col = conn.database(DATABASE).collection(SCHED_COL);
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

impl Handler<DbCreateUser> for DbServer {
    type Result = ResponseFuture<Result<bool, Error>>;
    fn handle(&mut self, msg: DbCreateUser, _ctx: &mut Self::Context) -> Self::Result {
        let DbCreateUser(user) = msg;
        let conn = self.handle.clone().expect("handle must exist");
        Box::pin(async move {
            let col = conn.database(DATABASE).collection(SCHED_COL);

            let modif = UpdateModifications::Document(doc! {
                "$setOnInsert": doc! { "username": user.clone(), "courses": [] }
            });
            let opt = UpdateOptions::builder().upsert(true).build();
            let update_result = col
                .update_one(doc! { "username": user.clone() }, modif, opt)
                .await?;

            Ok(update_result.upserted_id.is_none())
        })
    }
}

impl Handler<DbUpdateSchedule> for DbServer {
    type Result = ResponseFuture<Result<bool, Error>>;
    fn handle(&mut self, msg: DbUpdateSchedule, _ctx: &mut Self::Context) -> Self::Result {
        let DbUpdateSchedule(user, sched) = msg;
        let conn = self.handle.clone().expect("handle must exist");
        Box::pin(async move {
            let col = conn.database(DATABASE).collection(SCHED_COL);

            let modif = UpdateModifications::Document(doc! {
                "$set": doc! {"courses": to_bson(&sched.courses).unwrap() }
            });
            let update_result = col
                .update_one(doc! { "username": user }, modif, None)
                .await?;

            Ok(update_result.modified_count == 1)
        })
    }
}

impl SystemService for DbServer {}
impl Supervised for DbServer {}
