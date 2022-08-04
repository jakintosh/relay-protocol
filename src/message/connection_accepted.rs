use crate::message::{Payload, ProtocolId, ProtocolKey};
use crate::{Node, PeerAddress, ProtocolMessage, ProtocolResponse};

/*

    If we are receiving this message, it means that someone accepted a
    connection that we initiated. They are telling us what their shorthand
    key is, as well as sending us a payload that we can optionally use to
    verify the sender. We should be delegating all of this information to
    the specified protocol to confirm or deny the connection. If confirmed,
    the protocol is expected to send a "Connection Confirmed" message to the
    other party, so that Relay can know what *our* shorthand is.

*/
pub fn handle(
    node: &Node,
    address: PeerAddress,
    protocol: ProtocolId,
    key: ProtocolKey,
    payload: Payload,
) {
    let response = node.relay_message(
        &protocol,
        ProtocolMessage::ConnectionAccepted {
            address,
            bytes: payload,
        },
    );

    handle_response(response);
}

fn handle_response(response: ProtocolResponse) -> Option<ProtocolMessage> {
    match response {
        _ => None,
    }
}
