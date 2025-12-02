use std::{
    io::Cursor,
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use bytes::BytesMut;
use minecrust_protocol::{
    Deserialize, Serialize,
    packets::v773::{
        HandshakingIncoming, StatusIncoming, StatusOutgoing,
        outgoing::{PongResponse, StatusResponse},
    },
    read_packet_length,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::TcpListener,
    spawn,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::trace!("creating listener");
    let listener = TcpListener::bind(SocketAddr::new(
        IpAddr::from_str("127.0.0.1").expect("ip to be parsed"),
        25565,
    ))
    .await
    .expect("listener to bind to 127.0.0.1:25565");

    tracing::info!("listening on 127.0.0.1:25565");

    loop {
        let Ok((mut socket, addr)) = listener.accept().await else {
            tracing::debug!("error while accepting connection");
            continue;
        };

        tracing::info!(?addr, "client connected");
        spawn(async move {
            let (reader, writer) = socket.split();
            let mut buffered_reader = BufReader::new(reader);
            let mut buffered_writer = BufWriter::new(writer);

            enum Status {
                Handshaking,
                StatusRequest,
                PingRequest,
                Done,
            }

            let mut current_status = Status::Handshaking;

            loop {
                let packet_length = read_packet_length(&mut buffered_reader).await.unwrap();
                let mut packet_bytes = BytesMut::zeroed(packet_length);
                let bytes = buffered_reader.read_exact(&mut packet_bytes).await.unwrap();
                tracing::trace!(bytes, "read bytes into buffer");

                let mut cursor = Cursor::new(&packet_bytes);
                let mut response_package_bytes = Vec::new();

                match current_status {
                    Status::Handshaking => {
                        match HandshakingIncoming::deserialize(&mut cursor).unwrap() {
                            HandshakingIncoming::Intention(packet) => {
                                tracing::debug!(?packet, "received handshake");
                                current_status = Status::StatusRequest;
                            }
                        }
                    }
                    Status::StatusRequest => {
                        match StatusIncoming::deserialize(&mut cursor).unwrap() {
                            StatusIncoming::StatusRequest(_) => {
                                tracing::debug!("received status request");

                                let response = StatusOutgoing::StatusResponse(StatusResponse(
                                    r#"
{
    "version": {
        "name": "1.21.10",
        "protocol": 773
    },
    "description": {
        "text": "Hello, world!"
    },
    "enforcesSecureChat": false
}
                                    "#
                                    .to_string(),
                                ));
                                response.serialize(&mut response_package_bytes).unwrap();

                                current_status = Status::PingRequest;
                            }
                            _ => tracing::error!("status packets out of order"),
                        }
                    }
                    Status::PingRequest => {
                        match StatusIncoming::deserialize(&mut cursor).unwrap() {
                            StatusIncoming::PingRequest(packet) => {
                                tracing::debug!("received ping request");

                                let response = StatusOutgoing::PongResponse(PongResponse {
                                    timestamp: packet.timestamp,
                                });
                                response.serialize(&mut response_package_bytes).unwrap();

                                current_status = Status::Done;
                            }
                            _ => tracing::error!("status packets out of order"),
                        }
                    }
                    Status::Done => {
                        tracing::debug!("done handling connection");
                        break;
                    }
                };

                buffered_writer
                    .write_all(&response_package_bytes)
                    .await
                    .unwrap();
                buffered_writer.flush().await.unwrap();
            }
        });
    }
}
