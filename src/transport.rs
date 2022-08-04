pub(crate) mod router;
pub(crate) mod tcp;
pub(crate) mod udp;

use bytes::Bytes;
use std::net::SocketAddr;
use std::os::unix::net::SocketAddr as UnixSocketAddr;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Clone, Debug)]
pub enum TransportProtocol {
    Datagram,
    Stream,
}

#[derive(Clone, Debug)]
pub enum PeerAddress {
    Unix {
        addr: UnixSocketAddr,
        protocol: TransportProtocol,
    },
    Internet {
        addr: SocketAddr,
        protocol: TransportProtocol,
    },
}

#[derive(Clone, Debug)]
pub struct Message {
    pub addr: PeerAddress,
    pub payload: Bytes,
}

pub(crate) type TransportTx = UnboundedSender<Message>;
pub(crate) type TransportRx = UnboundedReceiverStream<Message>;

struct TransportFrame {
    addr: SocketAddr,
    bytes: Bytes,
}

type FrameTx = UnboundedSender<TransportFrame>;
type FrameRx = UnboundedReceiver<TransportFrame>;
