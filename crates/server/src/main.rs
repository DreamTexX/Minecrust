use std::{
    io::Cursor,
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use bytes::BytesMut;
use minecrust_protocol::{
    Deserialize, Serialize, VarInt,
    packets::v773::{
        HandshakingIncoming, StatusIncoming, StatusOutgoing,
        outgoing::{PongResponse, StatusResponse},
    },
};
use tokio::{
    io::{
        AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter,
    },
    net::TcpListener,
    spawn,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

enum Status {
    Handshaking,
    StatusRequest,
    PingRequest,
    Done,
}

/// Method to get the packet length. This method efficiently bridges between the sync
/// [`Deserialize`] API used in [`VarInt`] and the [`tokio::io::AsyncRead`] used in [`BufReader`].
///
/// Minecraft packets are prefixed with a [`VarInt`] to describe the length of the coming packet.
/// These kind of data types (as defined in [`minecrust_protocol`]) use a sync [`Deserialize`] API
/// with a [`std::io::Read`] because normally there is no case and need for some async API. Except
/// for parsing the packet size, where the length of the incoming byte stream is not yet known.
///
/// To prevent changing the existing API or building a second method only for [`VarInt`] to read
/// from an async byte stream with no known size we peek into the contents of the [`BufReader`] with
/// [`BufReader::fill_buf`]. This returns an array of bytes already read, which we can pass down to
/// the [`VarInt`] [`Deserialize`] Function. After reading the packet size we advance the
/// [`BufReader`] with the consumed bytes and continue parsing the packet.
async fn parse_packet_length<R: AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
) -> minecrust_protocol::Result<usize> {
    let mut peeked_bytes;
    loop {
        // peek the input stream
        peeked_bytes = reader.fill_buf().await?;
        if peeked_bytes.is_empty() {
            // EOF
            return Ok(0);
        }

        if peeked_bytes.len() >= 2 {
            // Smallest Minecraft Packet contains two bytes, one for the length and one for a 0x00
            // id with no data
            break;
        }
    }

    // todo: we somehow need to handle the case that not enough bytes where read and a wrong packet
    // size got parsed...
    let packet_length = VarInt::deserialize(&mut peeked_bytes)?;

    // remove used bytes for packet length from reader
    reader.consume(packet_length.consumed());

    Ok(*packet_length as usize)
}

async fn handle_connection<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
    mut reader: BufReader<R>,
    mut writer: BufWriter<W>,
) -> minecrust_protocol::Result<()> {
    let mut current_status = Status::Handshaking;

    loop {
        let packet_length = parse_packet_length(&mut reader).await?;
        if packet_length == 0 {
            tracing::trace!("no more packet received");
            return Ok(());
        }

        let mut packet_bytes = BytesMut::zeroed(packet_length);
        let bytes = reader.read_exact(&mut packet_bytes).await?;
        tracing::trace!(bytes, "read bytes into buffer");

        let mut cursor = Cursor::new(&packet_bytes);
        let mut response_package_bytes = Vec::new();

        match current_status {
            Status::Handshaking => match HandshakingIncoming::deserialize(&mut cursor)? {
                HandshakingIncoming::Intention(packet) => {
                    tracing::debug!(?packet, "received handshake");
                    current_status = Status::StatusRequest;
                }
            },
            Status::StatusRequest => match StatusIncoming::deserialize(&mut cursor)? {
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
                    response.serialize(&mut response_package_bytes)?;

                    current_status = Status::PingRequest;
                }
                _ => tracing::error!("status packets out of order"),
            },
            Status::PingRequest => match StatusIncoming::deserialize(&mut cursor)? {
                StatusIncoming::PingRequest(packet) => {
                    tracing::debug!("received ping request");

                    let response = StatusOutgoing::PongResponse(PongResponse {
                        timestamp: packet.timestamp,
                    });
                    response.serialize(&mut response_package_bytes)?;

                    current_status = Status::Done;
                }
                _ => tracing::error!("status packets out of order"),
            },
            Status::Done => {
                tracing::debug!("done handling connection");
                break;
            }
        };

        writer.write_all(&response_package_bytes).await?;
        writer.flush().await?;
    }

    Ok(())
}

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
        let Ok((socket, addr)) = listener.accept().await else {
            tracing::debug!("error while accepting connection");
            continue;
        };
        tracing::info!(?addr, "client connected");

        let (read_half, writ_half) = socket.into_split();
        let reader = BufReader::new(read_half);
        let writer = BufWriter::new(writ_half);

        spawn(handle_connection(reader, writer));
    }
}
