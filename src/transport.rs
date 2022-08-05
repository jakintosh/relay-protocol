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
        address: UnixSocketAddr,
        protocol: TransportProtocol,
    },
    Internet {
        address: SocketAddr,
        protocol: TransportProtocol,
    },
}

#[derive(Clone, Debug)]
pub struct Message {
    pub address: PeerAddress,
    pub payload: Bytes,
}

pub(crate) type TransportTx = UnboundedSender<Message>;
pub(crate) type TransportRx = UnboundedReceiverStream<Message>;

struct TransportFrame {
    address: SocketAddr,
    bytes: Bytes,
}

type FrameTx = UnboundedSender<TransportFrame>;
type FrameRx = UnboundedReceiver<TransportFrame>;
