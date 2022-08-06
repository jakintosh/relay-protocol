use crate::message::{Payload, ProtocolId, ProtocolKey};
use crate::{Message, Node, PeerAddress};

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
    node: &mut Node,
    address: PeerAddress,
    protocol_id: ProtocolId,
    peer_key: ProtocolKey,
    payload: Payload,
) -> Option<Message> {
    // verify the payload with the protocol (also get its key)
    let (verification_payload, my_key) = match node.get_protocol(&protocol_id) {
        None => return None, // invalid protocol id
        Some(p) => {
            let key = p.key;
            let verification_payload = p
                .handler
                .verify_accepted_connection(address.clone(), payload);

            (verification_payload, key)
        }
    };

    // if verification was successful, it should be "Some"
    let payload = match verification_payload {
        Some(payload) => payload,
        None => return None, // connection denied
    };

    // insert peer key into protocol's peer_keys table
    match node.get_protocol_mut(&protocol_id) {
        None => return None, // invalid protocol id
        Some(p) => p.peer_keys.insert(address, peer_key),
    };

    // everything worked out, return our key
    Some(Message::ConnectionConfirmed {
        protocol: protocol_id,
        key: my_key,
        payload,
    })
}
