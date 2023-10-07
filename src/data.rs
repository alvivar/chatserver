use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use fastwebsockets::Frame;
use tokio::sync::{mpsc::UnboundedSender, RwLock};

pub type Tx = UnboundedSender<Message>;
pub type SharedState = Arc<RwLock<State>>;

pub struct State {
    pub clients: HashMap<SocketAddr, Tx>,
}

impl State {
    pub async fn broadcast(&self, sender: &SocketAddr, msg: Message) {
        for (addr, tx) in self.clients.iter() {
            if addr != sender {
                tx.send(msg.clone()).unwrap();
            }
        }
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum Message {
    Text(String),
    Binary(Vec<u8>),
    Pong(Vec<u8>),
    Close(u16, String), // Code, Reason
}

impl Message {
    pub fn to_frame(&self) -> Frame {
        match self {
            Message::Text(text) => Frame::text(text.as_bytes().into()),
            Message::Binary(data) => Frame::binary(data.as_slice().into()),
            Message::Pong(data) => Frame::pong(data.as_slice().into()),
            Message::Close(code, reason) => Frame::close(*code, reason.as_bytes()),
        }
    }
}
