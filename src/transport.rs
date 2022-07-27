pub(crate) mod router;
pub(crate) mod tcp;
pub(crate) mod udp;

use bytes::Bytes;
use std::net::SocketAddr;
use std::os::unix::net::SocketAddr as UnixSocketAddr;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

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

pub struct TransportMessage {
    pub addr: PeerAddress,
    pub bytes: Bytes,
}

pub type TransportTx = UnboundedSender<TransportMessage>;
pub type TransportRx = UnboundedReceiver<TransportMessage>;

struct RawTransportMessage {
    addr: SocketAddr,
    bytes: Bytes,
}

type RawTransportTx = UnboundedSender<RawTransportMessage>;
type RawTransportRx = UnboundedReceiver<RawTransportMessage>;
