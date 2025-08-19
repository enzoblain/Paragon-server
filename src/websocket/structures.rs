use axum::extract::ws::Message;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex}
};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

pub type Clients = Arc<Mutex<HashMap<Uuid, UnboundedSender<Message>>>>;

pub struct Role (pub ClientRole);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientRole {
    Sender,
    Receiver
}