use super::{MessageId, ProtocolId};
use crate::{Node, NodeMessage, NodeResponse, PeerAddress};

/*
    If we are receiving this message, we need to make sure that we recently
    sent a negotiable message with that message id. if we didn't, then we
    ignore. if we did, then we should grab that message id, understand the
    semantics of that message, and then figure out how to send a new message
    with the chosen protocol? who should figure this out? relay should not
    be in the business of doing that. actually, i think that this just
    illustrates the fact that i'm thinking about this in the wrong way. Relay
    is not an app, but a protocol; any "application" can implement the protocol,
    and it just lets them speak multiple languages with each other. so, in the
    case of this library, i should pass this request back to the implementor to
    handle.

    so what needs to happen in here? i want to surface to the user of the library
    that we recieved a negotiation request they need to be able to know:
        1) who is asking
        2) which request they are responding to?
            a) how do we manage this in the first place? should the
               implementer keep track? should this library keep track?
*/
pub fn handle(node: &Node, address: PeerAddress, message_id: MessageId, proposal: ProtocolId) {
    let response = node.notify_delegate(NodeMessage::ProtocolNegotiated {
        address,
        message_id,
        proposal,
    });
    handle_response(response);
}

fn handle_response(response: NodeResponse) -> Option<NodeMessage> {
    match response {
        _ => None,
    }
}
