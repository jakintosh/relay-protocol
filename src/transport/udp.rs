use super::{FrameRx, FrameTx, TransportFrame};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::UdpSocket;

pub(super) async fn listen(
    port: u16,
    buffer_size: usize,
    in_frame_tx: FrameTx,
    out_frame_rx: FrameRx,
) {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let socket = UdpSocket::bind(address).await.expect("couldn't bind udp");
    let sender = Arc::new(socket);
    let listener = sender.clone();
    println!("{} Listening", addr_str(address));

    tokio::select! {
        () = self::handle_incoming_data(buffer_size, listener, in_frame_tx) => {},
        () = self::handle_outgoing_data(out_frame_rx, sender) => {},
    };
}

async fn handle_incoming_data(buffer_size: usize, listener: Arc<UdpSocket>, in_frame_tx: FrameTx) {
    let mut buf = vec![0u8; buffer_size];
    loop {
        match listener.recv_from(&mut buf).await {
            Ok((len, addr)) => {
                let buf = Vec::from(&buf[0..len]);
                match in_frame_tx.send(TransportFrame {
                    addr,
                    bytes: buf.into(),
                }) {
                    Ok(()) => println!("{} Received {} bytes", addr_str(addr), len),
                    Err(err) => println!("{} Received {} bytes ERR: {}", addr_str(addr), len, err),
                }
            }
            Err(_) => println!("UDP Receive failure"),
        }
    }
}

async fn handle_outgoing_data(mut out_frame_rx: FrameRx, sender: Arc<UdpSocket>) {
    while let Some(message) = out_frame_rx.recv().await {
        let TransportFrame { addr, bytes } = message;
        match sender.send_to(bytes.as_ref(), addr).await {
            Ok(len) => println!("{} Send {} bytes", addr_str(addr), len),
            Err(_) => println!("{} Send failure", addr_str(addr)),
        }
    }
}

fn addr_str(addr: SocketAddr) -> String {
    format!("ip{{{}}}->udp{{{}}}", addr.ip(), addr.port(),)
}
