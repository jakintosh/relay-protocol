use super::{RawTransportMessage, RawTransportRx, RawTransportTx};

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use std::{collections::HashMap, net::SocketAddr};
use tokio::{
    net::TcpListener,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub(super) async fn listen(
    port: u16,
    transport_msg_tx: RawTransportTx,
    transport_msg_rx: RawTransportRx,
) {
    let (t, r) = mpsc::unbounded_channel();

    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(address).await.expect("couldn't bind tcp");
    println!(
        "ip{{{}}}->tcp{{{}}} Listening",
        address.ip(),
        address.port()
    );

    // kick off processes
    tokio::select! {
        _ = accept_connections(listener, transport_msg_tx, t.clone()) => {}
        _ = handle_connection_events(r) => {},
        _ = relay_outgoing_bytes(transport_msg_rx, t) => {},
    };
}

enum ConnectionEvent {
    New {
        addr: SocketAddr,
        channel: UnboundedSender<Bytes>,
    },
    Send {
        addr: SocketAddr,
        bytes: Bytes,
    },
}
type ConnEventTx = UnboundedSender<ConnectionEvent>;
type ConnEventRx = UnboundedReceiver<ConnectionEvent>;

async fn accept_connections(
    listener: TcpListener,
    inbound_tx: RawTransportTx,
    conn_event_tx: ConnEventTx,
) {
    loop {
        let conn_event_tx = conn_event_tx.clone();
        match listener.accept().await {
            Ok((tcp, addr)) => {
                let inbound_tx = inbound_tx.clone();
                tokio::spawn(async move {
                    let (mut sink, mut stream) =
                        Framed::new(tcp, LengthDelimitedCodec::new()).split();

                    // relay all inbound data from tcp stream
                    tokio::spawn(async move {
                        while let Some(frame) = stream.next().await {
                            match frame {
                                Ok(bytes) => {
                                    println!(
                                        "ip{{{}}}->tcp{{{}}} Received {} bytes",
                                        addr.ip(),
                                        addr.port(),
                                        bytes.len()
                                    );
                                    let msg = RawTransportMessage {
                                        addr,
                                        bytes: bytes.freeze(),
                                    };
                                    if let Err(_) = inbound_tx.send(msg) {}
                                }
                                Err(err) => {
                                    println!(
                                        "ip{{{}}}->tcp{{{}}} Receive Error: {}",
                                        addr.ip(),
                                        addr.port(),
                                        err
                                    );
                                }
                            }
                        }
                    });

                    // hand off all outbound data to tcp stream
                    let (outbound_tx, mut outbound_rx) = mpsc::unbounded_channel::<Bytes>();
                    tokio::spawn(async move {
                        while let Some(bytes) = outbound_rx.recv().await {
                            let count = bytes.len();
                            match sink.send(bytes).await {
                                Ok(()) => {
                                    println!(
                                        "ip{{{}}}->tcp{{{}}} Sent {} bytes",
                                        addr.ip(),
                                        addr.port(),
                                        count
                                    );
                                }
                                Err(err) => {
                                    println!(
                                        "ip{{{}}}->tcp{{{}}} Send Error: {} ",
                                        addr.ip(),
                                        addr.port(),
                                        err
                                    );
                                }
                            }
                        }
                    });

                    // fire event for new connection
                    let new_conn_event = ConnectionEvent::New {
                        addr,
                        channel: outbound_tx,
                    };
                    if let Err(_) = conn_event_tx.send(new_conn_event) {
                        println!("couldn't register tcp connection");
                    }
                });
            }
            Err(e) => println!("couldn't accept client: {:?}", e),
        }
    }
}

async fn handle_connection_events(mut connection_event_rx: ConnEventRx) {
    let mut map = HashMap::new();
    while let Some(event) = connection_event_rx.recv().await {
        match event {
            ConnectionEvent::New { addr, channel } => {
                map.insert(addr, channel);
            }
            ConnectionEvent::Send { addr, bytes } => match map.get(&addr) {
                Some(channel) => if let Err(_) = channel.send(bytes) {},
                None => todo!(),
            },
        }
    }
}

async fn relay_outgoing_bytes(
    mut transport_msg_rx: RawTransportRx,
    connection_event_tx: ConnEventTx,
) {
    while let Some(message) = transport_msg_rx.recv().await {
        let RawTransportMessage { addr, bytes } = message;
        if let Err(_) = connection_event_tx.send(ConnectionEvent::Send { addr, bytes }) {
            println!("couldn't relay outgoing bytes for {}", addr);
        };
    }
}
