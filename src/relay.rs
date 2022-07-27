use crate::transport::{router::Router, PeerAddress, TransportMessage};
use bytes::Bytes;
use futures::StreamExt;
use std::collections::HashMap;

type MessageId = u8;
type PageCount = u8;
type ProtocolId = Vec<u8>;
type ProtocolKey = u8;
type Payload = Vec<u8>;
type PayloadMask = u8;

pub enum ExternalMessage {
    NegotiableMessage {
        message_id: MessageId,
        page_count: PageCount,
        proposals: Vec<ProtocolId>,
        payload_mask: PayloadMask,
        payload: Payload,
    },
    NegotiatedProtocolChoice {
        message_id: MessageId,
        proposal: ProtocolId,
    },
    NegotiationFailed {
        message_id: MessageId,
        page_count: PageCount,
    },
    ConnectionAccepted {
        protocol: ProtocolId,
        key: ProtocolKey,
        payload: Payload,
    },
    ConnectionConfirmed {
        protocol: ProtocolId,
        key: ProtocolKey,
        payload: Payload,
    },
    ConnectionMessage {
        key: ProtocolKey,
        payload: Payload,
    },
}
impl From<Bytes> for ExternalMessage {
    fn from(_: Bytes) -> Self {
        todo!()
    }
}
impl From<ExternalMessage> for Bytes {
    fn from(_: ExternalMessage) -> Self {
        todo!()
    }
}

pub enum InternalMessage {
    MessageReceived {
        address: PeerAddress,
        payload: Payload,
    },
}

pub enum InternalResponse {
    MessageAcknowledged,
    ConnectionAccepted {
        address: PeerAddress,
        payload: Payload,
    },
    ConnectionClosed {
        address: PeerAddress,
        payload: Payload,
    },
    UnregisteredProtocolId,
}

pub trait ProtocolHandler {
    fn receive_message(&self, message: InternalMessage) -> InternalResponse;
}

struct RegisteredProtocol {
    handler: Box<dyn ProtocolHandler + Sync>,
    key: ProtocolKey,
    peer_keys: HashMap<PeerAddress, ProtocolKey>,
}

pub struct RelayNode {
    last_protocol_key: u8,
    registered_protocols_by_id: HashMap<ProtocolId, RegisteredProtocol>,
}

impl RelayNode {
    pub fn new() -> RelayNode {
        RelayNode {
            last_protocol_key: 0,
            registered_protocols_by_id: HashMap::new(),
        }
    }

    pub fn register_handler(&mut self, id: ProtocolId, handler: Box<dyn ProtocolHandler + Sync>) {
        // Remove registered handler, if it exists
        let (key, peer_keys) = match self.registered_protocols_by_id.remove(&id) {
            // If it existed, extract and reuse existing key/peer_keys
            Some(RegisteredProtocol { key, peer_keys, .. }) => (key, peer_keys),
            // If not, create new key/peer_keys
            None => (self.get_next_key(), HashMap::new()),
        };

        let protocol = RegisteredProtocol {
            handler,
            key,
            peer_keys,
        };
        self.registered_protocols_by_id.insert(id, protocol);
    }

    pub async fn listen(&mut self) {
        let port = 27850;
        let buffer_size = 512;
        let mut router = Router::new(port, buffer_size);

        while let Some(message) = router.next().await {
            let TransportMessage { addr, bytes } = message;
            println!("addr{{{:?}}} Received {} bytes", addr, bytes.len());
            if let Some((addr, message)) = self.handle_external_message(addr, bytes.into()) {
                router.send(TransportMessage {
                    addr,
                    bytes: message.into(),
                });
            }
        }
    }

    fn handle_external_message(
        &mut self,
        address: PeerAddress,
        message: ExternalMessage,
    ) -> Option<(PeerAddress, ExternalMessage)> {
        let return_addr = address.clone();
        let response = match message {
            ExternalMessage::NegotiableMessage {
                message_id,
                page_count,
                proposals,
                payload_mask,
                payload,
            } => self.handle_negotiated_message(
                address,
                message_id,
                page_count,
                proposals,
                payload_mask,
                payload,
            ),
            ExternalMessage::NegotiatedProtocolChoice {
                message_id,
                proposal,
            } => self.handle_negotiated_protocol_choice(address, message_id, proposal),
            ExternalMessage::NegotiationFailed {
                message_id,
                page_count,
            } => self.handle_negotiation_failed(address, message_id, page_count),
            ExternalMessage::ConnectionAccepted {
                protocol,
                key,
                payload,
            } => self.handle_connection_accepted(address, protocol, key, payload),
            ExternalMessage::ConnectionConfirmed {
                protocol,
                key,
                payload,
            } => self.handle_connection_confirmed(address, protocol, key, payload),
            ExternalMessage::ConnectionMessage { key, payload } => {
                self.handle_connection_message(address, key, payload)
            }
        };

        match response {
            Some(msg) => Some((return_addr, msg)),
            None => None,
        }
    }

    fn handle_negotiated_message(
        &mut self,
        address: PeerAddress,
        message_id: MessageId,
        page_count: PageCount,
        proposals: Vec<ProtocolId>,
        payload_mask: PayloadMask,
        payload: Payload,
    ) -> Option<ExternalMessage> {
        for (index, protocol) in proposals.iter().enumerate() {
            // we don't support this protocol, skip it
            if !self.has_registered_protocol(protocol) {
                continue;
            }

            // we support the protocol, but need a different payload
            if !RelayNode::is_mask_bit_set(payload_mask, index) {
                let proposal = protocol.clone();
                return Some(ExternalMessage::NegotiatedProtocolChoice {
                    message_id,
                    proposal,
                });
            }

            // we support the protocol and the payload, relay it and handle the response
            let relay_message = InternalMessage::MessageReceived { address, payload };
            let relay_response = self.relay_message(protocol, relay_message);
            return match relay_response {
                InternalResponse::ConnectionAccepted { address, payload } => todo!(),
                InternalResponse::ConnectionClosed { address, payload } => todo!(),
                _ => None,
            };
        }

        // none of the proposals are supported, negotation failed
        return Some(ExternalMessage::NegotiationFailed {
            message_id,
            page_count,
        });
    }
    fn handle_negotiated_protocol_choice(
        &self,
        address: PeerAddress,
        message_id: MessageId,
        proposal: ProtocolId,
    ) -> Option<ExternalMessage> {
        None
    }
    fn handle_negotiation_failed(
        &self,
        address: PeerAddress,
        message_id: MessageId,
        page_count: PageCount,
    ) -> Option<ExternalMessage> {
        None
    }
    fn handle_connection_accepted(
        &self,
        address: PeerAddress,
        proposal: ProtocolId,
        key: ProtocolKey,
        payload: Vec<u8>,
    ) -> Option<ExternalMessage> {
        None
    }
    fn handle_connection_confirmed(
        &self,
        address: PeerAddress,
        proposal: ProtocolId,
        key: ProtocolKey,
        payload: Vec<u8>,
    ) -> Option<ExternalMessage> {
        None
    }
    fn handle_connection_message(
        &self,
        address: PeerAddress,
        key: ProtocolKey,
        payload: Payload,
    ) -> Option<ExternalMessage> {
        None
    }

    fn relay_message(&mut self, id: &ProtocolId, message: InternalMessage) -> InternalResponse {
        match self.get_protocol(id) {
            Some(protocol) => protocol.handler.receive_message(message),
            None => InternalResponse::UnregisteredProtocolId,
        }
    }

    fn has_registered_protocol(&self, id: &ProtocolId) -> bool {
        self.registered_protocols_by_id.contains_key(id)
    }
    fn get_next_key(&mut self) -> ProtocolKey {
        self.last_protocol_key += 1;
        return self.last_protocol_key;
    }
    fn get_protocol(&mut self, id: &ProtocolId) -> Option<&RegisteredProtocol> {
        self.registered_protocols_by_id.get(id)
    }
    fn is_mask_bit_set(mask: PayloadMask, index: usize) -> bool {
        match index < 8 {
            true => (1 << index) & mask != 0,
            false => false,
        }
    }
}
