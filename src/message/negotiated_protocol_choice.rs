use super::{MessageId, ProtocolId};
use crate::{Message, Node, PeerAddress};

/*
    If we are receiving this message, we need to make sure that we recently
    sent a negotiable message with that message id. This is handled by the
    delegate for the relay node.

*/
pub fn handle(
    node: &Node,
    address: PeerAddress,
    message_id: MessageId,
    proposal: ProtocolId,
) -> Option<Message> {
    // delegate the message
    let delegate = node.clone_delegate();
    delegate.handle_negotiated_protocol(address, message_id, proposal);

    None
}
