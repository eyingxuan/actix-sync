use crate::actors::dbserver::models::Schedule;
use actix::prelude::*;
use mongodb::error::Error;

#[derive(Clone, Message)]
#[rtype(result = "Result<Option<Schedule>, Error>")]
pub struct DbGetSchedule(pub String);
