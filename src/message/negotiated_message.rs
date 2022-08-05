use super::{Message, MessageId, PageCount, Payload, PayloadMask, ProtocolId};
use crate::{Node, PeerAddress, ProtocolMessage, ProtocolResponse};

pub fn handle(
    node: &Node,
    address: PeerAddress,
    message_id: MessageId,
    page_count: PageCount,
    proposals: Vec<ProtocolId>,
    payload_mask: PayloadMask,
    payload: Payload,
) -> Option<Message> {
    for (index, protocol) in proposals.iter().enumerate() {
        // we don't support this protocol, skip it
        if !node.has_registered_protocol(protocol) {
            continue;
        }

        // we support the protocol, but need a different payload
        if !is_mask_bit_set(payload_mask, index) {
            let proposal = protocol.clone();
            return Some(Message::NegotiatedProtocolChoice {
                message_id,
                proposal,
            });
        }

        // we support the protocol and the payload, relay it and handle the response
        let bytes = payload;
        let protocol_message = ProtocolMessage::MessageReceived { address, bytes };
        let response = node.relay_message(protocol, protocol_message);
        let relay_message = handle_response(response);
        return relay_message;
    }

    // none of the proposals are supported, negotation failed
    return Some(Message::NegotiationFailed {
        message_id,
        page_count,
    });
}

fn handle_response(response: ProtocolResponse) -> Option<Message> {
    match response {
        ProtocolResponse::RelayMessages(msgs) => todo!(),
        ProtocolResponse::ConnectionAccepted(msgs) => todo!(),
        ProtocolResponse::ConnectionClosed(msgs) => todo!(),
        _ => None,
    }
}

fn is_mask_bit_set(mask: PayloadMask, index: usize) -> bool {
    match index < 8 {
        true => (1 << index) & mask != 0,
        false => false,
    }
}
