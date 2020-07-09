use crate::message::dbmessage::*;
use actix::prelude::*;
use actix::utils::IntervalFunc;
use crdts::orswot::Orswot;
use models::*;
use mongodb::bson::{doc, from_bson, to_bson, Bson};
use mongodb::error::Error;
use mongodb::options::{UpdateModifications, UpdateOptions};
use mongodb::Client;
use std::collections::HashMap;
use std::env::var;
use std::time::Duration;

pub mod models;

const DATABASE: &'static str = "sync";
const SCHED_COL: &'static str = "sched";
const COURSE_COL: &'static str = "courses";

#[derive(Default)]
pub struct DbServer {
    handle: Option<Client>,
    cache: HashMap<String, (bool, Orswot<String, u8>)>,
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

        IntervalFunc::new(
            Duration::from_secs(3),
            |act: &mut Self, ctx: &mut Self::Context| {
                for (username, (updated, crdt)) in &act.cache {
                    if !updated {
                        let new_sched = Schedule {
                            username: username.clone(),
                            courses: crdt.clone().read().val.into_iter().collect::<Vec<String>>(),
                        };
                        ctx.address()
                            .recipient()
                            .send(DbUpdateSchedule(username.clone(), new_sched))
                            .into_actor(act)
                            .then(|_, _, _| fut::ready(()))
                            .spawn(ctx);
                    }
                }

                for (_, tp) in &mut act.cache {
                    tp.0 = true;
                }
            },
        )
        .finish()
        .spawn(ctx);
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
                "$set": doc! {"courses": to_bson(&sched).unwrap() }
            });
            let update_result = col
                .update_one(doc! { "username": user }, modif, None)
                .await?;

            Ok(update_result.modified_count == 1)
        })
    }
}

impl Handler<DbUpdateCache> for DbServer {
    type Result = ();
    fn handle(&mut self, msg: DbUpdateCache, _ctx: &mut Self::Context) -> Self::Result {
        let DbUpdateCache(user, sched) = msg;
        self.cache.insert(user, (false, sched));
    }
}

impl SystemService for DbServer {}
impl Supervised for DbServer {}
