use super::{
    tcp, udp, FrameRx, FrameTx, Message, PeerAddress, TransportFrame, TransportProtocol,
    TransportRx, TransportTx,
};
use tokio::sync::mpsc::unbounded_channel;

pub(crate) struct Router {
    tcp_out_frame_tx: FrameTx,
    udp_out_frame_tx: FrameTx,
}
impl Router {
    pub fn new(port: u16, buffer_size: usize) -> (Router, TransportRx) {
        let (tcp_transport_tx, transport_rx) = unbounded_channel();
        let udp_transport_tx = tcp_transport_tx.clone();

        let (tcp_in_frame_tx, tcp_in_frame_rx) = unbounded_channel();
        let (tcp_out_frame_tx, tcp_out_frame_rx) = unbounded_channel();
        tokio::spawn(tcp::listen(port, tcp_in_frame_tx, tcp_out_frame_rx));
        tokio::spawn(Router::handle_tcp_incoming(
            tcp_in_frame_rx,
            tcp_transport_tx,
        ));

        let (udp_in_frame_tx, udp_in_frame_rx) = unbounded_channel();
        let (udp_out_frame_tx, udp_out_frame_rx) = unbounded_channel();
        tokio::spawn(udp::listen(
            port,
            buffer_size,
            udp_in_frame_tx,
            udp_out_frame_rx,
        ));
        tokio::spawn(Router::handle_udp_incoming(
            udp_in_frame_rx,
            udp_transport_tx,
        ));

        let router = Router {
            tcp_out_frame_tx,
            udp_out_frame_tx,
        };
        let transport_msg_stream = transport_rx.into();

        (router, transport_msg_stream)
    }

    pub fn send(&self, message: Message) {
        let Message {
            address,
            payload: bytes,
        } = message;
        match address {
            PeerAddress::Unix {
                address: _,
                protocol,
            } => match protocol {
                TransportProtocol::Datagram => panic!("unix sockets not supported yet"),
                TransportProtocol::Stream => panic!("unix sockets not supported yet"),
            },
            PeerAddress::Internet { address, protocol } => match protocol {
                TransportProtocol::Datagram => {
                    if let Err(_) = self
                        .udp_out_frame_tx
                        .send(TransportFrame { address, bytes })
                    {
                        // can't send
                    }
                }
                TransportProtocol::Stream => {
                    if let Err(_) = self
                        .tcp_out_frame_tx
                        .send(TransportFrame { address, bytes })
                    {
                        // can't send
                    }
                }
            },
        }
    }

    async fn handle_tcp_incoming(mut tcp_recv_rx: FrameRx, transport_tx: TransportTx) {
        while let Some(tcp_message) = tcp_recv_rx.recv().await {
            let TransportFrame {
                address,
                bytes: payload,
            } = tcp_message;
            let address = PeerAddress::Internet {
                address,
                protocol: TransportProtocol::Stream,
            };
            if let Err(_) = transport_tx.send(Message { address, payload }) {
                // can't send any more transport messages
            }
        }
    }

    async fn handle_udp_incoming(mut udp_recv_rx: FrameRx, transport_tx: TransportTx) {
        while let Some(udp_message) = udp_recv_rx.recv().await {
            let TransportFrame {
                address,
                bytes: payload,
            } = udp_message;
            let address = PeerAddress::Internet {
                address,
                protocol: TransportProtocol::Datagram,
            };
            if let Err(_) = transport_tx.send(Message { address, payload }) {
                // can't send any more transport messages
            }
        }
    }
}
