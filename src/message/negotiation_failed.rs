use super::{MessageId, PageCount};
use crate::{Message, Node, PeerAddress};

pub fn handle(
    node: &Node,
    address: PeerAddress,
    message_id: MessageId,
    page_count: PageCount,
) -> Option<Message> {
    node.get_delegate()
        .handle_negotiation_failure(address, message_id, page_count);

    None
}
