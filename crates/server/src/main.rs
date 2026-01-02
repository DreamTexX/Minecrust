use std::{
    error::Error,
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::{
        Arc, LazyLock,
        atomic::{AtomicUsize, Ordering},
    },
};

use arc_swap::ArcSwap;
use etcd_client::{GetOptions, KeyValue, WatchOptions};
use minecrust_protocol::datatype::Intent;
use tokio::{
    net::{TcpListener, TcpStream},
    signal,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::Instrument;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    codec::PlainCodec,
    connection::{Connection, handshake},
    handler::{Handler, StatusHandler},
};

mod codec;
mod connection;
mod handler;

static NEXT_CONNECTION_ID: AtomicUsize = AtomicUsize::new(0usize);
static SERVER_STATE: LazyLock<ServerState> = LazyLock::new(|| ServerState {
    description: ArcSwap::from_pointee(
        r##"
        {
            "text": "This is a Minecrust Server!",
            "type": "text",
            "color": "#f5d545"
        }
    "##
        .to_string(),
    ),
});

#[derive(Debug)]
struct ServerState {
    description: ArcSwap<String>,
}

async fn client_loop(connection: Connection<PlainCodec>) -> minecrust_protocol::Result<()> {
    match connection.intent {
        Intent::Status => StatusHandler::new(connection).handle().await?,
        Intent::Login => {}
        Intent::Transfer => {}
    }

    Ok(())
}

async fn handle_connection(stream: TcpStream, addr: SocketAddr) {
    let id = NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed);
    let span = tracing::trace_span!("connection", connection_id = id);

    match handshake(id, stream, addr).await {
        Ok(connection) => {
            if let Err(err) = client_loop(connection).instrument(span).await {
                tracing::warn!(?err, "connection closed with error")
            }
            tracing::info!("connection closed");
        }
        Err(err) => {
            tracing::warn!(?err, "handshake failed")
        }
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cancellation_token = CancellationToken::new();
    let tracker = TaskTracker::new();
    tracker.spawn(run_etcd_listener(cancellation_token.clone()));
    tracker.spawn(run_server(tracker.clone(), cancellation_token.clone()));

    tracing::trace!("waiting for shutdown signal");
    signal::ctrl_c().await?;
    tracing::trace!("shutdown signal received");

    cancellation_token.cancel();
    tracing::trace!("cancellation issued");

    tracker.close();
    tracing::trace!("task tracker closed");
    tracker.wait().await;
    tracing::trace!("all tasks completed");

    Ok(())
}

async fn parse_etcd_kv(kv: &KeyValue) -> Result<(), etcd_client::Error> {
    let Ok(key) = kv.key_str() else {
        tracing::debug!("unparsable key received");
        return Ok(());
    };
    match key {
        "server/description" => {
            let description: String = String::from_utf8_lossy(kv.value()).into_owned();
            tracing::debug!(key, description, "updated server motd");
            SERVER_STATE.description.store(Arc::new(description));
        }
        key => {
            tracing::warn!(key, "unknown key received by watcher: {}", key);
        }
    }
    Ok(())
}

async fn run_etcd_listener(
    cancellation_token: CancellationToken,
) -> Result<(), etcd_client::Error> {
    let mut client = etcd_client::Client::connect(["localhost:2379"], None).await?;

    let (_watcher, mut stream) = client
        .watch(
            "server",
            Some(WatchOptions::new().with_prefix().with_prev_key()),
        )
        .await?;

    let snapshot = client
        .get("server", Some(GetOptions::new().with_prefix()))
        .await?;
    for kv in snapshot.kvs() {
        parse_etcd_kv(kv).await?;
    }

    loop {
        tokio::select! {
            biased;

            _ = cancellation_token.cancelled() => break,

            maybe_message = stream.message() => {
                if let Some(message) = maybe_message? {
                    if message.canceled() {
                        tracing::debug!("watcher canceled");
                        break;
                    }
                    if message.created() {
                        tracing::debug!(watch_id = message.watch_id(), "watcher created");
                    }

                    for event in message.events() {
                        if let Some(kv) = event.kv() {
                            parse_etcd_kv(kv).await?;
                        }
                    }
                }
            }
        }
    }

    tracing::trace!("stopping etcd watcher");
    Ok(())
}

async fn run_server(
    tracker: TaskTracker,
    cancellation_token: CancellationToken,
) -> Result<(), tokio::io::Error> {
    let listener = TcpListener::bind(SocketAddr::new(
        IpAddr::from_str("127.0.0.1").expect("ip to be parsed"),
        25565,
    ))
    .await?;
    tracing::info!("listening on 127.0.0.1:25565");

    loop {
        tokio::select! {
            biased;

            _ = cancellation_token.cancelled() => break,

            accept = listener.accept() => {
                let Ok((stream, addr)) = accept else {
                    tracing::debug!("error while accepting connection");
                    continue;
                };

                tracker.spawn(handle_connection(stream, addr));
            }
        }
    }

    tracing::trace!("closing listener");
    Ok(())
}
