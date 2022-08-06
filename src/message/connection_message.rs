use crate::message::{Payload, ProtocolKey};
use crate::{Message, Node, PeerAddress};

/*

*/
pub fn handle(
    node: &Node,
    address: PeerAddress,
    key: ProtocolKey,
    payload: Payload,
) -> Option<Message> {
    let id = match node.get_protocol_id(key) {
        None => return None, // invalid protocol key
        Some(id) => id,
    };

    let protocol = match node.get_protocol(&id) {
        None => return None, // invalid protocol id
        Some(p) => p,
    };

    protocol.handler.handle_message(address, payload);

    None
}
