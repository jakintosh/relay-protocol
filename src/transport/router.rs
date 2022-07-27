use super::{
    tcp, udp, PeerAddress, RawTransportMessage, RawTransportRx, RawTransportTx, TransportMessage,
    TransportProtocol, TransportRx, TransportTx,
};
use futures::Stream;
use tokio::sync::mpsc::unbounded_channel;

pub(crate) struct Router {
    tcp_send_tx: RawTransportTx,
    udp_send_tx: RawTransportTx,
    transport_rx: TransportRx,
}
impl Router {
    pub fn new(port: u16, buffer_size: usize) -> Router {
        let (transport_tx, transport_rx) = unbounded_channel();

        let (tcp_recv_tx, tcp_recv_rx) = unbounded_channel();
        let (tcp_send_tx, tcp_send_rx) = unbounded_channel();
        tokio::spawn(tcp::listen(port, tcp_recv_tx, tcp_send_rx));
        tokio::spawn(Router::handle_tcp_recv(tcp_recv_rx, transport_tx.clone()));

        let (udp_recv_tx, udp_recv_rx) = unbounded_channel();
        let (udp_send_tx, udp_send_rx) = unbounded_channel();
        tokio::spawn(udp::listen(port, buffer_size, udp_recv_tx, udp_send_rx));
        tokio::spawn(Router::handle_udp_recv(udp_recv_rx, transport_tx.clone()));

        Router {
            tcp_send_tx,
            udp_send_tx,
            transport_rx,
        }
    }

    pub fn send(&self, message: TransportMessage) {
        let TransportMessage { addr, bytes } = message;
        match addr {
            PeerAddress::Unix { addr, protocol } => match protocol {
                TransportProtocol::Datagram => panic!("unix sockets not supported yet"),
                TransportProtocol::Stream => panic!("unix sockets not supported yet"),
            },
            PeerAddress::Internet { addr, protocol } => match protocol {
                TransportProtocol::Datagram => {
                    if let Err(_) = self.udp_send_tx.send(RawTransportMessage { addr, bytes }) {
                        // can't send
                    }
                }
                TransportProtocol::Stream => {
                    if let Err(_) = self.tcp_send_tx.send(RawTransportMessage { addr, bytes }) {
                        // can't send
                    }
                }
            },
        }
    }

    async fn handle_tcp_recv(mut tcp_recv_rx: RawTransportRx, transport_tx: TransportTx) {
        while let Some(msg_in) = tcp_recv_rx.recv().await {
            let RawTransportMessage { addr, bytes } = msg_in;
            let addr = PeerAddress::Internet {
                addr,
                protocol: TransportProtocol::Stream,
            };
            println!("router tcp recv addr: {{{:?}}}", addr);
            if let Err(_) = transport_tx.send(TransportMessage { addr, bytes }) {
                // can't send any more transport messages
            }
        }
    }

    async fn handle_udp_recv(mut udp_recv_rx: RawTransportRx, transport_tx: TransportTx) {
        while let Some(msg_in) = udp_recv_rx.recv().await {
            let RawTransportMessage { addr, bytes } = msg_in;
            let addr = PeerAddress::Internet {
                addr,
                protocol: TransportProtocol::Datagram,
            };
            println!("router udp recv addr: {{{:?}}}", addr);
            if let Err(_) = transport_tx.send(TransportMessage { addr, bytes }) {
                // can't send any more transport messages
            }
        }
    }
}

impl Stream for Router {
    type Item = TransportMessage;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.transport_rx.try_recv() {
            Ok(message) => std::task::Poll::Ready(Some(message)),
            Err(err) => match err {
                tokio::sync::mpsc::error::TryRecvError::Empty => std::task::Poll::Pending,
                tokio::sync::mpsc::error::TryRecvError::Disconnected => {
                    std::task::Poll::Ready(None)
                }
            },
        }
    }
}
