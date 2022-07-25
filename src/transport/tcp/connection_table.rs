use super::NetworkFrameWrite;
use bytes::Bytes;
use futures_sink::Sink;
use std::collections::HashMap;
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    oneshot,
};

pub(crate) enum Message {
    Insert {
        sink: NetworkFrameWrite,
        id_tx: oneshot::Sender<usize>,
    },
    Remove(usize),
    Send {
        id: usize,
        bytes: Bytes,
    },
}

pub(crate) type MessageTx = UnboundedSender<Message>;
pub(crate) type MessageRx = UnboundedReceiver<Message>;

pub(crate) async fn listen(mut message_rx: MessageRx) {
    let mut id = 0usize;
    let mut map = HashMap::new();

    while let Some(message) = message_rx.recv().await {
        match message {
            Message::Insert { sink, id_tx } => {
                let sink_id = id;
                id = (std::num::Wrapping(id) + std::num::Wrapping(1)).0;
                println!("inserting sink with id {}", sink_id);
                map.insert(sink_id, sink);
                match id_tx.send(sink_id) {
                    Ok(_) => {}
                    Err(_) => println!("failed to notify about insert"),
                }
            }
            Message::Remove(sink_id) => {
                println!("removing sink with id {}", sink_id);
                map.remove(&sink_id);
            }
            Message::Send { id: sink_id, bytes } => {
                println!("sending to sink with id {}", sink_id);
                let sink = match map.get_mut(&sink_id) {
                    Some(s) => s,
                    None => {
                        println!("can't send response, sink not found");
                        continue;
                    }
                };
                // if let Err(_) = sink.send(bytes.into()).await {};
            }
        }
    }
    message_rx.close();
}
