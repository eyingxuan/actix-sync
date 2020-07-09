use crate::actors::dbserver::models::Schedule;
use actix::prelude::*;
use crdts::orswot::Orswot;
use mongodb::error::Error;

// parameters: username
#[derive(Clone, Message)]
#[rtype(result = "Result<Option<Schedule>, Error>")]
pub struct DbGetSchedule(pub String);

// parameters: username
#[derive(Clone, Message)]
#[rtype(result = "Result<bool, Error>")]
pub struct DbCreateUser(pub String);

// parameters: username, schedule
#[derive(Clone, Message)]
#[rtype(result = "Result<bool, Error>")]
pub struct DbUpdateSchedule(pub String, pub Schedule);

#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct DbUpdateCache(pub String, pub Orswot<String, u8>);
