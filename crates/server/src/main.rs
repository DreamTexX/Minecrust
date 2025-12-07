use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::{
        Arc, LazyLock,
        atomic::{AtomicUsize, Ordering},
    },
};

use arc_swap::ArcSwap;
use etcd_client::{GetOptions, KeyValue, WatchOptions};
use tokio::{
    net::{TcpListener, TcpStream},
    spawn,
};
use tracing::Instrument;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{connection::Connection, handler::Handler};

mod connection;
mod handler;

static CONNECTION_COUNT: AtomicUsize = AtomicUsize::new(0usize);
static SERVER_STATE: LazyLock<ServerState> = LazyLock::new(|| ServerState {
    description: ArcSwap::from_pointee(
        r##"
        {
            "text": "This is a minecraft Server!",
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

fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
) -> impl Future<Output = minecrust_protocol::Result<()>> {
    let id = CONNECTION_COUNT.fetch_add(1, Ordering::Relaxed);
    let span = tracing::trace_span!("connection", connection_id = id);

    async move {
        let connection = Connection::new(id, stream, addr).await?;
        let mut handler = connection.into_handler();
        handler.handle().await?;

        tracing::info!("closing connection");

        Ok(())
    }
    .instrument(span)
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

async fn run_etcd_listener() -> Result<(), etcd_client::Error> {
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

    while let Some(message) = stream.message().await? {
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

    Ok(())
}

async fn run_server() -> Result<(), tokio::io::Error> {
    let listener = TcpListener::bind(SocketAddr::new(
        IpAddr::from_str("127.0.0.1").expect("ip to be parsed"),
        25565,
    ))
    .await?;

    tracing::info!("listening on 127.0.0.1:25565");
    loop {
        let Ok((stream, addr)) = listener.accept().await else {
            tracing::debug!("error while accepting connection");
            continue;
        };

        spawn(handle_connection(stream, addr));
    }
}
