use crate::message::{Payload, ProtocolKey};
use crate::{Message, Node, PeerAddress};

/*

    If we're getting this message, it means we have an established
    connection with this address, and that we should forward it to
    the given protocol.

*/
pub fn handle(
    node: &Node,
    address: PeerAddress,
    key: ProtocolKey,
    payload: Payload,
) -> Option<Message> {
    // get id from the key
    let id = match node.get_protocol_id(key) {
        None => return None, // invalid protocol key
        Some(id) => id,
    };

    // relay the message
    match node.get_protocol(&id) {
        None => return None, // invalid protocol id
        Some(p) => p.handler.handle_message(address, payload),
    };

    None
}
