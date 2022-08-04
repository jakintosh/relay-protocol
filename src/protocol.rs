use crate::message::ProtocolKey;
use crate::transport::{Message as TransportMessage, PeerAddress};
use std::collections::HashMap;

#[derive(Debug)]
pub enum Message {
    MessageReceived {
        address: PeerAddress,
        bytes: Vec<u8>,
    },
    ConnectionAccepted {
        address: PeerAddress,
        bytes: Vec<u8>,
    },
}

#[derive(Debug)]
pub enum Response {
    MessageAcknowledged,
    RelayMessages(Vec<TransportMessage>),
    ConnectionAccepted(Vec<TransportMessage>),
    ConnectionClosed(Vec<TransportMessage>),
    UnregisteredProtocolId,
}

pub trait Handler {
    fn receive_message(&self, message: Message) -> Response;
}

pub(crate) struct Protocol {
    pub(crate) handler: Box<dyn Handler>,
    pub(crate) key: ProtocolKey,
    pub(crate) peer_keys: HashMap<PeerAddress, ProtocolKey>,
}
