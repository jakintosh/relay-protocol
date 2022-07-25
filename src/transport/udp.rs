use super::{RawTransportMessage, RawTransportRx, RawTransportTx};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::UdpSocket;

pub(super) async fn listen(
    port: u16,
    buffer_size: usize,
    transport_msg_tx: RawTransportTx,
    mut transport_msg_rx: RawTransportRx,
) {
    // bind and "split" the socket
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let socket = UdpSocket::bind(address).await.expect("couldn't bind udp");
    let sender = Arc::new(socket);
    let listener = sender.clone();
    println!(
        "ip{{{}}}->udp{{{}}} Listening",
        address.ip(),
        address.port()
    );

    // send outgoing data
    tokio::spawn(async move {
        while let Some(message) = transport_msg_rx.recv().await {
            let RawTransportMessage { addr, bytes } = message;
            match sender.send_to(bytes.as_ref(), addr).await {
                Ok(len) => {
                    println!(
                        "ip{{{}}}->udp{{{}}} Send {} bytes",
                        address.ip(),
                        address.port(),
                        len
                    );
                }
                Err(_) => {
                    println!(
                        "ip{{{}}}->udp{{{}}} Send failure",
                        address.ip(),
                        address.port()
                    );
                }
            }
        }
    });

    // receive incoming data
    let mut buf = vec![0u8; buffer_size];
    loop {
        match listener.recv_from(&mut buf).await {
            Ok((len, addr)) => {
                let buf = Vec::from(&buf[0..len]);
                match transport_msg_tx.send(RawTransportMessage {
                    addr,
                    bytes: buf.into(),
                }) {
                    Ok(()) => {
                        println!(
                            "ip{{{}}}->udp{{{}}} Received {} bytes",
                            address.ip(),
                            address.port(),
                            len
                        );
                    }
                    Err(err) => {
                        println!(
                            "ip{{{}}}->udp{{{}}} Received {} bytes but ERROR: {}",
                            address.ip(),
                            address.port(),
                            len,
                            err
                        );
                    }
                }
            }
            Err(_) => {
                println!(
                    "ip{{{}}}->udp{{{}}} Receive failure",
                    address.ip(),
                    address.port()
                );
            }
        }
    }
}
