use crate::message::ProtocolKey;
use crate::transport::PeerAddress;
use crate::Payload;
use std::collections::HashMap;

pub trait Handler {
    fn handle_message(&self, address: PeerAddress, payload: Payload);
    fn verify_accepted_connection(&self, address: PeerAddress, payload: Payload)
        -> Option<Payload>;
    fn verify_confirmed_connection(&self, address: PeerAddress, payload: Payload) -> bool;
    fn verify_closed_connection(&self, address: PeerAddress, payload: Payload) -> bool;
}

pub(crate) struct Protocol {
    pub(crate) handler: Box<dyn Handler>,
    pub(crate) key: ProtocolKey,
    pub(crate) peer_keys: HashMap<PeerAddress, ProtocolKey>,
}
