use crate::message::{self, Message, MessageId, PageCount, ProtocolId, ProtocolKey};
use crate::protocol::Protocol;
use crate::transport::{router::Router, Message as TransportMessage, TransportRx};
use crate::{PeerAddress, ProtocolHandler};
use futures::StreamExt;
use std::collections::HashMap;

pub trait Delegate {
    fn handle_negotiated_protocol(
        &self,
        address: PeerAddress,
        message_id: MessageId,
        protocol_id: ProtocolId,
    );
    fn handle_negotiation_failure(
        &self,
        address: PeerAddress,
        message_id: MessageId,
        page_count: PageCount,
    );
}

pub struct Node {
    router: Router,
    message_stream: TransportRx,
    last_key: u8,
    delegate: Box<dyn Delegate>,
    protocols_by_id: HashMap<ProtocolId, Protocol>,
    ids_by_key: HashMap<ProtocolKey, ProtocolId>,
}

impl Node {
    pub fn new(delegate: Box<dyn Delegate>) -> Node {
        let port = 27850;
        let buffer_size = 512;
        let (router, message_stream) = Router::new(port, buffer_size);

        Node {
            router,
            message_stream,
            last_key: 0,
            delegate,
            protocols_by_id: HashMap::new(),
            ids_by_key: HashMap::new(),
        }
    }

    pub fn register_protocol(&mut self, id: ProtocolId, handler: Box<dyn ProtocolHandler>) {
        // remove registered handler
        //   if it existed, extract and reuse existing key/peer_keys
        //   if not, create new key/peer_keys
        let (key, peer_keys) = match self.protocols_by_id.remove(&id) {
            Some(protocol) => (protocol.key, protocol.peer_keys.clone()),
            None => (self.get_next_key(), HashMap::new()),
        };

        // construct a new protocol and insert it into the table
        let protocol = Protocol {
            handler,
            key,
            peer_keys,
        };
        self.protocols_by_id.insert(id, protocol);
    }

    pub async fn listen(&mut self) {
        while let Some(transport_message) = self.message_stream.next().await {
            let TransportMessage { address, payload } = transport_message;
            println!("RELAY: {:?} Received {} bytes", address, payload.len());
            let relay_message = match payload.try_into() {
                Ok(msg) => msg,
                Err(err) => {
                    println!("couldn't deserialize relay message: {}", err);
                    continue;
                }
            };

            message::handle(self, address, relay_message);
        }
    }

    pub fn send(&mut self, address: PeerAddress, message: Message) {
        let payload = match message.try_into() {
            Ok(bytes) => bytes,
            Err(_) => todo!(),
        };
        self.router.send(TransportMessage { address, payload });
    }

    pub(crate) fn get_protocol(&self, id: &ProtocolId) -> Option<&Protocol> {
        match self.protocols_by_id.get(id) {
            Some(protocol) => Some(protocol),
            None => None,
        }
    }
    pub(crate) fn get_protocol_mut(&mut self, id: &ProtocolId) -> Option<&mut Protocol> {
        match self.protocols_by_id.get_mut(id) {
            Some(protocol) => Some(protocol),
            None => None,
        }
    }

    pub(crate) fn borrow_delegate(&self) -> &Box<dyn Delegate> {
        &self.delegate
    }

    pub(crate) fn get_protocol_id(&self, key: ProtocolKey) -> Option<&ProtocolId> {
        self.ids_by_key.get(&key)
    }

    fn get_next_key(&mut self) -> ProtocolKey {
        self.last_key += 1;
        return self.last_key;
    }
}
