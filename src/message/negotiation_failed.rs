use super::{MessageId, PageCount};
use crate::{Node, NodeMessage, NodeResponse, PeerAddress};

pub fn handle(node: &Node, address: PeerAddress, message_id: MessageId, page_count: PageCount) {
    let response = node.notify_delegate(NodeMessage::ProtocolNegotiationFailed {
        address,
        message_id,
        page_count,
    });

    handle_response(response);
}

fn handle_response(response: NodeResponse) -> Option<NodeMessage> {
    match response {
        _ => None,
    }
}
