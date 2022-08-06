pub(crate) mod router;
pub(crate) mod tcp;
pub(crate) mod udp;

use bytes::Bytes;
use std::hash::Hash;
use std::net::SocketAddr;
use std::os::unix::net::SocketAddr as UnixSocketAddr;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Clone, Debug, PartialEq, Hash)]
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
impl Hash for PeerAddress {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            PeerAddress::Unix { address, protocol } => {
                address.as_pathname().hash(state);
                protocol.hash(state);
            }
            PeerAddress::Internet { address, protocol } => {
                address.hash(state);
                protocol.hash(state);
            }
        }
    }
}
impl PartialEq for PeerAddress {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                PeerAddress::Unix {
                    address: l_address,
                    protocol: l_protocol,
                },
                PeerAddress::Unix {
                    address: r_address,
                    protocol: r_protocol,
                },
            ) => l_address.as_pathname() == r_address.as_pathname() && l_protocol == r_protocol,
            (
                PeerAddress::Internet {
                    address: l_address,
                    protocol: l_protocol,
                },
                PeerAddress::Internet {
                    address: r_address,
                    protocol: r_protocol,
                },
            ) => l_address == r_address && l_protocol == r_protocol,
            _ => false,
        }
    }
}
impl Eq for PeerAddress {}

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
