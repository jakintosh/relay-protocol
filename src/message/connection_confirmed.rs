use crate::message::{Payload, ProtocolId, ProtocolKey};
use crate::{Message, Node, PeerAddress, ProtocolMessage, ProtocolResponse};

/*

*/
pub fn handle(
    node: &Node,
    address: PeerAddress,
    protocol: ProtocolId,
    key: ProtocolKey,
    payload: Payload,
) -> Option<Message> {
    let response = node.relay_message(
        &protocol,
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
