use crate::message::{Payload, ProtocolId, ProtocolKey};
use crate::{Message, Node, PeerAddress};

/*

*/
pub fn handle(
    node: &Node,
    address: PeerAddress,
    protocol: ProtocolId,
    key: ProtocolKey,
    payload: Payload,
) -> Option<Message> {
    None
}
