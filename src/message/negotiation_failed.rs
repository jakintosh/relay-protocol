use super::{MessageId, PageCount};
use crate::{Message, Node, PeerAddress};

/*

    If we are receiving this message, it's because we sent a negotiated message
    and the receiver wasn't able to handle it. We need to notify the delegate,
    who will determine if it will try again with new proposals.

*/
pub fn handle(
    node: &Node,
    address: PeerAddress,
    message_id: MessageId,
    page_count: PageCount,
) -> Option<Message> {
    // delegate the message
    let delegate = node.clone_delegate();
    delegate.handle_negotiation_failure(address, message_id, page_count);

    None
}
