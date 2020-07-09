use actix::prelude::*;
use crdts::orswot::{Op, Orswot};

#[derive(Clone, Message)]
#[rtype(result = "Result<Option<(Orswot<String, u8>, u8)>, std::convert::Infallible>")]
pub struct InitiateSync(pub Recipient<ScheduleMessage>, pub String);

#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct UpdateSchedule(
    pub String,
    pub Recipient<ScheduleMessage>,
    pub Op<String, u8>,
);

#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct ScheduleMessage(pub Op<String, u8>);
