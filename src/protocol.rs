use crate::message::ProtocolKey;
use crate::transport::PeerAddress;
use std::collections::HashMap;

pub trait Handler {
    fn handle_message(&self, address: PeerAddress, payload: Vec<u8>);
    fn handle_accepted_connection(&self, address: PeerAddress, payload: Vec<u8>) -> bool;
    fn handle_closed_connection(&self, address: PeerAddress, payload: Vec<u8>);
}

pub(crate) struct Protocol {
    pub(crate) handler: Box<dyn Handler>,
    pub(crate) key: ProtocolKey,
    pub(crate) peer_keys: HashMap<PeerAddress, ProtocolKey>,
}
