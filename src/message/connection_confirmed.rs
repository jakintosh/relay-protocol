use crate::message::{Payload, ProtocolId, ProtocolKey};
use crate::{Message, Node, PeerAddress};

/*

    If we're receiving this message, it means we let the sender know that
    we've accepted their connection, and they have confirmed it and know
    our key, and that this is their key. we need to get validation from the
    relayed protocol that this message is legit, and if so we need to add
    this key to our protocol's peer key table.

*/
pub fn handle(
    node: &mut Node,
    address: PeerAddress,
    protocol_id: ProtocolId,
    peer_key: ProtocolKey,
    payload: Payload,
) -> Option<Message> {
    // ask the protocol to verify this message
    match node.get_protocol(&protocol_id) {
        None => return None, // invalid protocol id
        Some(p) => {
            let address = address.clone();
            if !p.handler.verify_confirmed_connection(address, payload) {
                return None; // failed protocol verification
            };
        }
    };

    // insert peer key into protocol's peer_keys table
    match node.get_protocol_mut(&protocol_id) {
        None => return None, // invalid protocol id
        Some(p) => p.peer_keys.insert(address, peer_key),
    };

    // everything worked out, no further work
    None
}
