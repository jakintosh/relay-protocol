use super::{Message, MessageId, PageCount, Payload, PayloadMask, ProtocolId};
use crate::{Node, PeerAddress};

pub fn handle(
    node: &Node,
    address: PeerAddress,
    message_id: MessageId,
    page_count: PageCount,
    proposals: Vec<ProtocolId>,
    payload_mask: PayloadMask,
    payload: Payload,
) -> Option<Message> {
    for (index, id) in proposals.iter().enumerate() {
        let protocol = match node.get_protocol(id) {
            None => continue, // we don't support this protocol, skip it
            Some(p) => p,
        };

        // we support the protocol, but need a different payload
        if !is_mask_bit_set(payload_mask, index) {
            let proposal = id.clone();
            return Some(Message::NegotiatedProtocolChoice {
                message_id,
                proposal,
            });
        }

        // we support the protocol and the payload, relay it and handle the respons
        {
            let p = protocol.lock().unwrap();
            p.handler.handle_message(address, payload);
        }

        return None;
    }

    // none of the proposals are supported, negotation failed
    return Some(Message::NegotiationFailed {
        message_id,
        page_count,
    });
}

fn is_mask_bit_set(mask: PayloadMask, index: usize) -> bool {
    match index < 8 {
        true => (1 << index) & mask != 0,
        false => false,
    }
}
