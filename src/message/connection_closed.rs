use crate::message::{Payload, ProtocolKey};
use crate::{Message, Node, PeerAddress};

/*

    If we are receiving this message, then a peer we have a connection with
    is asking to close the connection. We need to let the protocol validate
    the request, and if so, we remove that address from the protocol's peer
    keys table.

*/
pub fn handle(
    node: &mut Node,
    address: PeerAddress,
    key: ProtocolKey,
    payload: Payload,
) -> Option<Message> {
    // get the id from the key
    let id = match node.get_protocol_id(key) {
        None => return None, // invalid protocol key
        Some(id) => id.clone(),
    };

    // ask the protocol to verify this message
    let verified = match node.get_protocol(&id) {
        None => return None, // invalid protocol id
        Some(p) => p.handler.verify_closed_connection(address.clone(), payload),
    };
    if !verified {
        return None; // failed protocol verification
    }

    // remove (addr/peer_key) from protocol's peer_keys table
    match node.get_protocol_mut(&id) {
        None => return None, // invalid protocol id
        Some(p) => p.peer_keys.remove(&address),
    };

    None
}
