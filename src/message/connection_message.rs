use crate::message::{Payload, ProtocolKey};
use crate::{Message, Node, PeerAddress, ProtocolMessage, ProtocolResponse};

/*

*/
pub fn handle(
    node: &Node,
    address: PeerAddress,
    key: ProtocolKey,
    payload: Payload,
) -> Option<Message> {
    let protocol_id = match node.get_protocol_id(key) {
        Some(id) => id,
        None => {
            // someone sent a bad message
            return None;
        }
    };
    let response = node.relay_message(
        &protocol_id,
        ProtocolMessage::ConnectionAccepted {
            address,
            bytes: payload,
        },
    );

    handle_response(response)
}

fn handle_response(response: ProtocolResponse) -> Option<Message> {
    match response {
        _ => None,
    }
}
