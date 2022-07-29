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
pub(crate) struct TransportMessage {
    pub addr: PeerAddress,
    pub bytes: Bytes,
}

pub(crate) type TransportTx = UnboundedSender<TransportMessage>;
pub(crate) type TransportRx = UnboundedReceiverStream<TransportMessage>;

struct TransportFrame {
    addr: SocketAddr,
    bytes: Bytes,
}

type FrameTx = UnboundedSender<TransportFrame>;
type FrameRx = UnboundedReceiver<TransportFrame>;
