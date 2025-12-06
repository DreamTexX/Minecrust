use std::{
    io::Cursor,
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::{Arc, LazyLock},
};

use arc_swap::ArcSwap;
use bytes::BytesMut;
use etcd_client::WatchOptions;
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
use tracing::Instrument;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static SERVER_STATE: LazyLock<ServerState> = LazyLock::new(|| {
    ServerState {
        motd: ArcSwap::from_pointee("This is a MineCrust server!".to_string()),
    }
});

#[derive(Debug)]
struct ServerState {
    motd: ArcSwap<String>,
}

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
    let packet_length: VarInt = loop {
        // peek the input stream
        let mut peeked_bytes = reader.fill_buf().await?;
        if peeked_bytes.is_empty() {
            // EOF
            return Ok(0);
        }

        match VarInt::deserialize(&mut peeked_bytes) {
            Ok(value) => {
                break value;
            }
            Err(err) => match err {
                minecrust_protocol::Error::Io(err)
                    if std::io::ErrorKind::UnexpectedEof == err.kind() =>
                {
                    // Not enough bytes read to build var int
                    continue;
                }
                _ => return Err(err),
            },
        };
    };

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

                    let response = StatusOutgoing::StatusResponse(StatusResponse(format!(
                        r#"
                        {{
                            "version": {{
                                "name": "1.21.10",
                                "protocol": 773
                            }},
                            "description": {{
                                "text": "{}"
                            }},
                            "enforcesSecureChat": false
                        }}
                        "#,
                        SERVER_STATE.motd.load()
                    )));
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

    tokio::select! {
        server_result = run_server() => {
            if let Err(err) = server_result {
                tracing::error!("running server exited with error: {}", err);
            }
        },
        etcd_listener_result = run_etcd_listener() => {
            if let Err(err) = etcd_listener_result {
                tracing::error!("etcd listener exited with error: {}", err);
            }
        }
    }
}

async fn run_etcd_listener() -> Result<(), etcd_client::Error> {
    let mut client = etcd_client::Client::connect(["localhost:2379"], None).await?;

    let (_watcher, mut stream) = client.watch("server", Some(WatchOptions::new().with_prev_key().with_prefix())).await?;

    while let Some(message) = stream.message().await? {
        if message.canceled() {
            tracing::debug!("watcher canceled");
            break;
        }
        if message.created() {
            tracing::debug!(watch_id=message.watch_id(), "watcher created")
        }

        for event in message.events() {
            if let Some(kv) = event.kv() {
                let Ok(key) = kv.key_str() else {
                    tracing::debug!("unparsable key received");
                    continue;
                };
                match key {
                    "server/motd" => {
                        let motd: String = String::from_utf8_lossy(kv.value()).into_owned();
                        tracing::debug!(key, motd, "updated server motd");
                        SERVER_STATE.motd.store(Arc::new(motd));
                    }
                    key => {
                        tracing::warn!(key, "unknown key received by watcher: {}", key);
                    }
                }
            }
        }
    }

    Ok(())
}

async fn run_server() -> Result<(), tokio::io::Error> {
    tracing::trace!("creating listener");
    let listener = TcpListener::bind(SocketAddr::new(
        IpAddr::from_str("127.0.0.1").expect("ip to be parsed"),
        25565,
    ))
    .await?;
    let mut connection_count = 0usize;

    tracing::info!("listening on 127.0.0.1:25565");

    loop {
        let Ok((socket, addr)) = listener.accept().await else {
            tracing::debug!("error while accepting connection");
            continue;
        };
        let (read_half, writ_half) = socket.into_split();
        let reader = BufReader::new(read_half);
        let writer = BufWriter::new(writ_half);

        let connection_id = connection_count;
        connection_count = connection_count.wrapping_add(1usize);

        let span = tracing::trace_span!("connection", connection_id);
        span.in_scope(|| {
            tracing::info!(?addr, "client connected");
        });
        spawn(handle_connection(reader, writer).instrument(span));
    }
}
