use super::{FrameRx, FrameTx, TransportFrame};
use bytes::Bytes;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::{collections::HashMap, net::SocketAddr};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

enum ConnectionMessage {
    New {
        address: SocketAddr,
        bytes_tx: BytesTx,
    },
    Send {
        address: SocketAddr,
        bytes: Bytes,
    },
}

type ConnMsgTx = UnboundedSender<ConnectionMessage>;
type ConnMessageRx = UnboundedReceiver<ConnectionMessage>;

type BytesTx = UnboundedSender<Bytes>;
type BytesRx = UnboundedReceiver<Bytes>;

type SplitTcpStream = SplitStream<Framed<TcpStream, LengthDelimitedCodec>>;
type SplitTcpSink = SplitSink<Framed<TcpStream, LengthDelimitedCodec>, Bytes>;

pub(super) async fn listen(port: u16, in_frame_tx: FrameTx, out_frame_rx: FrameRx) {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(address).await.expect("couldn't bind tcp");
    println!("{} Listening", addr_str(address));

    let (conn_msg_tx, conn_msg_rx) = mpsc::unbounded_channel();
    tokio::select! {
        () = accept_connections(listener, in_frame_tx, conn_msg_tx.clone()) => {}
        () = process_connection_messages(conn_msg_rx) => {},
        () = relay_outgoing_bytes(out_frame_rx, conn_msg_tx) => {},
    };
}

async fn accept_connections(listener: TcpListener, in_frame_tx: FrameTx, conn_msg_tx: ConnMsgTx) {
    loop {
        match listener.accept().await {
            Ok((tcp, address)) => {
                let in_frame_tx = in_frame_tx.clone();

                let (sink, stream) = Framed::new(tcp, LengthDelimitedCodec::new()).split();
                let (bytes_tx, bytes_rx) = mpsc::unbounded_channel::<Bytes>();
                tokio::spawn(handle_tcp_stream(stream, address, in_frame_tx));
                tokio::spawn(handle_tcp_sink(sink, address, bytes_rx));

                if let Err(_) = conn_msg_tx.send(ConnectionMessage::New { address, bytes_tx }) {
                    println!("Error: Couldn't register new tcp connection");
                    println!("       Connection message receiver is closed");
                }
            }
            Err(err) => println!("transport::tcp couldn't accept client: {:?}", err),
        }
    }
}

async fn handle_tcp_stream(mut stream: SplitTcpStream, address: SocketAddr, in_frame_tx: FrameTx) {
    while let Some(frame) = stream.next().await {
        match frame {
            Ok(bytes) => {
                let len = bytes.len();
                let bytes = bytes.freeze();
                let msg = TransportFrame { address, bytes };
                match in_frame_tx.send(msg) {
                    Ok(()) => println!("{} Received {} bytes", addr_str(address), len),
                    Err(_) => {
                        println!("{} Error: Frame receiver is closed", addr_str(address));
                        return;
                    }
                }
            }
            Err(err) => println!("{} Receive Error: {}", addr_str(address), err),
        }
    }
}

async fn handle_tcp_sink(mut sink: SplitTcpSink, address: SocketAddr, mut bytes_rx: BytesRx) {
    while let Some(bytes) = bytes_rx.recv().await {
        let count = bytes.len();
        match sink.send(bytes).await {
            Ok(()) => println!("{} Sent {} bytes", addr_str(address), count),
            Err(err) => println!("{} Send Error: {} ", addr_str(address), err),
        }
    }
}

async fn process_connection_messages(mut conn_msg_rx: ConnMessageRx) {
    let mut map = HashMap::new();
    while let Some(message) = conn_msg_rx.recv().await {
        match message {
            ConnectionMessage::New { address, bytes_tx } => {
                map.insert(address, bytes_tx);
            }
            ConnectionMessage::Send { address, bytes } => match map.get(&address) {
                Some(bytes_tx) => match bytes_tx.send(bytes) {
                    Ok(()) => {}
                    Err(_) => {
                        println!("{} Error: Byte receiver is closed", addr_str(address));
                        map.remove(&address);
                    }
                },
                None => println!("{} Error: No registered byte receiver", addr_str(address)),
            },
        }
    }
}

async fn relay_outgoing_bytes(mut out_frame_tx: FrameRx, conn_msg_tx: ConnMsgTx) {
    while let Some(message) = out_frame_tx.recv().await {
        let TransportFrame { address, bytes } = message;
        if let Err(_) = conn_msg_tx.send(ConnectionMessage::Send { address, bytes }) {
            println!("Error: Connection message receiver is closed");
            return;
        };
    }
}

fn addr_str(address: SocketAddr) -> String {
    format!("ip{{{}}}->tcp{{{}}}", address.ip(), address.port(),)
}
