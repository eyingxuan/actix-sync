use crate::db::Schedule;
use actix::prelude::*;

// Message Parameters
// Sender: Client
// Receiver: Server
// 1: Address (receipient) of sender in order to subscribe to update events
// 2: Username of client
#[derive(Clone, Message)]
#[rtype(result = "Result<bool, std::convert::Infallible>")]
pub struct InitiateSync(pub Recipient<ScheduleMessage>, pub String);

// Message Parameters
// Sender: Client
// Receiver: Server
// 1: Username of client
// 2: Schedule to update
#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct UpdateSchedule(pub String, pub Schedule);

// Message Parameters
// Sender: Server
// Receiver: Client
// 1. Schedule updated on server
#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct ScheduleMessage(pub Schedule);
