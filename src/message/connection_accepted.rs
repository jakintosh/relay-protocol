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
    id: ProtocolId,
    peer_key: ProtocolKey,
    payload: Payload,
) -> Option<Message> {
    let (conn_confirmed, my_key) = match node.get_protocol(&id) {
        None => return None, // invalid protocol id
        Some(p) => {
            let key = p.key;
            let confirmed = !p
                .handler
                .handle_accepted_connection(address.clone(), payload);

            (confirmed, key)
        }
    };

    if !conn_confirmed {
        // connection denied
    }

    if !node.register_peer_key(address.clone(), &id, peer_key) {
        // failed to register peer key
    }

    // everything worked out, return our key
    node.send(
        address,
        Message::ConnectionConfirmed {
            protocol: id,
            key: my_key,
            payload: Vec::new(),
        },
    );

    None
}
