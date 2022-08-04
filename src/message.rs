use crate::{Node, PeerAddress};
use bytes::Bytes;
use serde::{Deserialize, Serialize};

pub mod connection_accepted;
pub mod negotiated_message;
pub mod negotiated_protocol_choice;
pub mod negotiation_failed;

// define types
pub(crate) type MessageId = u8;
pub(crate) type PageCount = u8;
pub(crate) type ProtocolId = Vec<u8>;
pub(crate) type ProtocolKey = u8;
pub(crate) type Payload = Vec<u8>;
pub(crate) type PayloadMask = u8;

// relay protocol messages
#[derive(Serialize, Deserialize)]
pub enum Message {
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
    ConnectionClosed {
        key: ProtocolKey,
        payload: Payload,
    },
    ConnectionMessage {
        key: ProtocolKey,
        payload: Payload,
    },
}
impl TryFrom<Bytes> for Message {
    type Error = String;

    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        match rmp_serde::from_slice(bytes.as_ref()) {
            Ok(msg) => Ok(msg),
            Err(err) => Err(format!("failed to deserialize bytes: {}", err)),
        }
    }
}
impl TryFrom<Message> for Bytes {
    type Error = String;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match rmp_serde::to_vec(&message) {
            Ok(bytes) => Ok(bytes.into()),
            Err(err) => Err(format!("failed to serialize message: {}", err)),
        }
    }
}

pub fn handle(
    node: &Node,
    address: PeerAddress,
    message: Message,
) -> Option<Vec<(PeerAddress, Message)>> {
    match message {
        Message::NegotiableMessage {
            message_id,
            page_count,
            proposals,
            payload_mask,
            payload,
        } => negotiated_message::handle(
            node,
            address,
            message_id,
            page_count,
            proposals,
            payload_mask,
            payload,
        ),
        Message::NegotiatedProtocolChoice {
            message_id,
            proposal,
        } => {
            negotiated_protocol_choice::handle(node, address, message_id, proposal);
            None
        }
        Message::NegotiationFailed {
            message_id,
            page_count,
        } => {
            negotiation_failed::handle(node, address, message_id, page_count);
            None
        }
        Message::ConnectionAccepted {
            protocol,
            key,
            payload,
        } => {
            connection_accepted::handle(node, address, protocol, key, payload);
            None
        }
        // ExternalMessage::ConnectionConfirmed {
        //     protocol,
        //     key,
        //     payload,
        // } => self::handle_connection_confirmed(node, address, protocol, key, payload),
        // ExternalMessage::ConnectionMessage { key, payload } => {
        //     self::handle_connection_message(node, address, key, payload)
        // }
        _ => None,
    };

    // match response {
    //     Some(msg) => Some((return_addr, msg)),
    //     None => None,
    // }
    None
}
