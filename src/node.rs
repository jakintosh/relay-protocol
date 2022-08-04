use crate::message::{self, MessageId, PageCount, ProtocolId, ProtocolKey};
use crate::protocol::Protocol;
use crate::transport::{router::Router, Message as TransportMessage};
use crate::{PeerAddress, ProtocolHandler, ProtocolMessage, ProtocolResponse};
use futures::StreamExt;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Message {
    ProtocolNegotiated {
        address: PeerAddress,
        message_id: MessageId,
        proposal: ProtocolId,
    },
    ProtocolNegotiationFailed {
        address: PeerAddress,
        message_id: MessageId,
        page_count: PageCount,
    },
}

#[derive(Debug)]
pub enum Response {
    None,
}

pub trait Delegate {
    fn receive_message(&self, message: Message) -> Response;
}

pub struct Node {
    last_key: u8,
    delegate: Box<dyn Delegate>,
    protocols_by_id: HashMap<ProtocolId, Protocol>,
}

impl Node {
    pub fn new(delegate: Box<dyn Delegate>) -> Node {
        Node {
            last_key: 0,
            delegate,
            protocols_by_id: HashMap::new(),
        }
    }

    pub fn register_protocol(&mut self, id: ProtocolId, handler: Box<dyn ProtocolHandler>) {
        // remove registered handler
        //   if it existed, extract and reuse existing key/peer_keys
        //   if not, create new key/peer_keys
        let (key, peer_keys) = match self.protocols_by_id.remove(&id) {
            Some(Protocol { key, peer_keys, .. }) => (key, peer_keys),
            None => (self.get_next_key(), HashMap::new()),
        };

        // construct a new protocol and insert it into the table
        self.protocols_by_id.insert(
            id,
            Protocol {
                handler,
                key,
                peer_keys,
            },
        );
    }

    pub async fn listen(&mut self) {
        let port = 27850;
        let buffer_size = 512;
        let (router, mut message_stream) = Router::new(port, buffer_size);

        while let Some(message) = message_stream.next().await {
            let TransportMessage {
                addr: address,
                payload,
            } = message;
            println!("RELAY: {:?} Received {} bytes", address, payload.len());
            let external_message = match payload.try_into() {
                Ok(msg) => msg,
                Err(err) => {
                    println!("couldn't deserialize external message: {}", err);
                    continue;
                }
            };
            if let Some(messages) = message::handle(self, address, external_message) {
                for (addr, message) in messages {
                    let payload = match message.try_into() {
                        Ok(bytes) => bytes,
                        Err(_) => todo!(),
                    };
                    router.send(TransportMessage { addr, payload });
                }
            }
        }
    }

    pub fn has_registered_protocol(&self, id: &ProtocolId) -> bool {
        self.protocols_by_id.contains_key(id)
    }

    pub(crate) fn relay_message(
        &self,
        id: &ProtocolId,
        message: ProtocolMessage,
    ) -> ProtocolResponse {
        match self.get_protocol(id) {
            Some(protocol) => protocol.handler.receive_message(message),
            None => ProtocolResponse::UnregisteredProtocolId,
        }
    }

    pub(crate) fn notify_delegate(&self, message: Message) -> Response {
        self.delegate.receive_message(message)
    }

    fn get_next_key(&mut self) -> ProtocolKey {
        self.last_key += 1;
        return self.last_key;
    }

    fn get_protocol(&self, id: &ProtocolId) -> Option<&Protocol> {
        self.protocols_by_id.get(id)
    }
}
