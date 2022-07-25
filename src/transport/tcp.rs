pub(crate) mod connection_table;

use super::{RawTransportRx, RawTransportTx};
use futures_util::StreamExt;
use std::net::SocketAddr;
use tokio::{
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener,
    },
    sync::{mpsc, oneshot},
};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

// type aliases
type NetworkFrameRead = FramedRead<OwnedReadHalf, LengthDelimitedCodec>;
type NetworkFrameWrite = FramedWrite<OwnedWriteHalf, LengthDelimitedCodec>;

pub(super) async fn listen(port: u16, bytes_tx: RawTransportTx, bytes_rx: RawTransportRx) {
    let (conn_msg_tx, conn_msg_rx) = mpsc::unbounded_channel();

    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(address).await.expect("couldn't bind tcp");
    println!(
        "ip{{{}}}->tcp{{{}}} LISTENING",
        address.ip(),
        address.port()
    );

    // kick off processes
    tokio::select! {
        _ = connection_table::listen(conn_msg_rx) => {}
        _ = accept_connections(listener, bytes_tx, conn_msg_tx.clone()) => {}
        _ = read_messages(bytes_rx, conn_msg_tx) => {},
    };
}

async fn accept_connections(
    listener: TcpListener,
    bytes_tx: RawTransportTx,
    conn_table_tx: connection_table::MessageTx,
) {
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                println!("accepted connection from {}", addr);

                // split and frame the tcp stream
                let (read, write) = stream.into_split();
                let stream = NetworkFrameRead::new(read, LengthDelimitedCodec::new());
                let sink = NetworkFrameWrite::new(write, LengthDelimitedCodec::new());

                // insert sink into connection table
                let (id_tx, id_rx) = oneshot::channel(); // create a oneshot value return channel
                let insert_message = connection_table::Message::Insert { sink, id_tx };
                if let Err(_) = conn_table_tx.send(insert_message) {
                    println!("connection table is closed, cannot accept any more connections");
                    return;
                }

                // wait for sink_id from sink table
                match id_rx.await {
                    Ok(sink_id) => {
                        let message_out = bytes_tx.clone();
                        tokio::spawn(poll_connection(
                            stream,
                            sink_id,
                            conn_table_tx.clone(),
                            message_out,
                        ));
                    }
                    Err(_) => {
                        println!(
                            "couldn't store connection sink for {}: dropping connection",
                            addr
                        );
                        continue;
                    }
                };
            }
            Err(e) => println!("couldn't accept client: {:?}", e),
        }
    }
}

async fn poll_connection(
    mut stream: NetworkFrameRead,
    sink_id: usize,
    conn_table_tx: connection_table::MessageTx,
    bytes_tx: RawTransportTx,
) {
    while let Some(frame) = stream.next().await {
        // decode the tcp frame
        let bytes = match frame {
            Ok(b) => b,
            Err(e) => {
                println!("tcp frame decode failure: {:?}", e);
                continue;
            }
        };

        // send the bytes elsewhere
    }

    // connection is closed, drop the sink
    let _unused_result = conn_table_tx.send(connection_table::Message::Remove(sink_id));
}

async fn read_messages(
    mut message_rx: RawTransportRx,
    sink_message_tx: connection_table::MessageTx,
) {
    while let Some(message) = message_rx.recv().await {}
}
