use crate::dbserver::DbServer;
use crate::message::clientmessage::*;
use crate::message::dbmessage::*;
use actix::prelude::*;
use crdts::orswot::Orswot;
use crdts::CmRDT;
use rand::prelude::*;
use std::collections::{HashMap, HashSet};

type ClientRecv = Recipient<ScheduleMessage>;

#[derive(Default)]
pub struct SyncServer {
    // unique IDs for each user
    client_ids: HashMap<String, HashSet<u8>>,
    // recv end for each user
    observers: HashMap<String, Vec<ClientRecv>>,
    // reference crdt for each user
    crdt_ref: HashMap<String, Orswot<String, u8>>,
}

impl SyncServer {
    // returns None when no set is created for the user
    fn gen_id(&self, username: &String) -> Option<u8> {
        let set = self.client_ids.get(username)?;
        if set.len() == 256 {
            panic!("way too many people are using my application");
        } else {
            loop {
                let rand = random::<u8>();
                if !(set.contains(&rand)) {
                    return Some(rand);
                }
            }
        }
    }
}

impl Handler<InitiateSync> for SyncServer {
    // TODO: consider adding proper error to differentiate connection errors versus invalid username
    type Result =
        ResponseActFuture<Self, Result<Option<(Orswot<String, u8>, u8)>, std::convert::Infallible>>;

    fn handle(&mut self, msg: InitiateSync, _ctx: &mut Self::Context) -> Self::Result {
        let InitiateSync(recp, username) = msg;
        let username = username.to_owned();

        if self.crdt_ref.contains_key(&username) {
            self.observers
                .get_mut(&username)
                .expect("key must be present in this branch")
                .push(recp);

            let unique_id: u8 = self
                .gen_id(&username)
                .expect("key must be present in this branch");

            self.client_ids
                .get_mut(&username)
                .expect("key must be present in this branch")
                .insert(unique_id);

            let cloned_crdt = self
                .crdt_ref
                .get(&username)
                .expect("key must be present in this branch")
                .clone();

            Box::new(async move { Ok(Some((cloned_crdt, unique_id))) }.into_actor(self))
        } else {
            Box::new(
                async move {
                    let res = DbServer::from_registry()
                        .send(DbGetSchedule(username.clone()))
                        .await;

                    // TODO: Add logging
                    match res {
                        Ok(Ok(Some(s))) => Some((s, username, recp)),
                        _ => None,
                    }
                }
                .into_actor(self)
                .map(|res, act, _| match res {
                    None => Ok(None),
                    Some((s, username, recp)) => {
                        let mut crdt = Orswot::new();
                        crdt.apply(crdt.add_all(s.courses, crdt.read_ctx().derive_add_ctx(1)));

                        let mut hashset = HashSet::new();
                        hashset.insert(1);

                        act.client_ids.insert(username.clone(), hashset);
                        act.observers.insert(username.clone(), vec![recp]);
                        act.crdt_ref.insert(username.clone(), crdt.clone());
                        Ok(Some((crdt, 1)))
                    }
                }),
            )
        }
    }
}

impl Handler<UpdateSchedule> for SyncServer {
    type Result = ();

    fn handle(&mut self, msg: UpdateSchedule, _ctx: &mut Self::Context) -> Self::Result {
        let UpdateSchedule(username, recp, op) = msg;

        // TODO: Send to DB for persistence

        self.crdt_ref
            .get_mut(&username)
            .expect("at least one session initiated sync")
            .apply(op.clone());
        for tx in self
            .observers
            .get(&username)
            .expect("at least one session initiated sync")
        {
            if tx != &recp {
                // TODO: Handle error when sending
                tx.do_send(ScheduleMessage(op.clone()))
                    .expect("handle later");
            }
        }
    }
}

impl Actor for SyncServer {
    type Context = Context<Self>;
}

impl SystemService for SyncServer {}
impl Supervised for SyncServer {}
