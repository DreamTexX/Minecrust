use std::{
    error::Error,
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use tokio::{net::TcpListener, signal};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod connection;
mod dispatcher;

async fn run_listener(
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
        tracing::trace!("waiting for connection");
        tokio::select! {
            biased;

            _ = cancellation_token.cancelled() => break,

            accept = listener.accept() => {
                let Ok((stream, remote_addr)) = accept else {
                    tracing::debug!("error while accepting connection");
                    continue;
                };
                let cancellation_token = cancellation_token.clone();
                tracing::trace!(?remote_addr, "connection accepted");

                tracker.spawn(connection::handle_connection(cancellation_token, stream));
            }
        }
    }

    tracing::trace!("closing listener");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let shutdown_signal = CancellationToken::new();
    let tracker = TaskTracker::new();
    // tracker.spawn(run_etcd_listener(cancellation_token.clone()));
    tracker.spawn(run_listener(tracker.clone(), shutdown_signal.clone()));

    tracing::trace!("waiting for shutdown signal");
    signal::ctrl_c().await?;
    tracing::trace!("shutdown signal received");

    shutdown_signal.cancel();
    tracing::trace!("cancellation issued");

    tracker.close();
    tracing::trace!("task tracker closed");
    tracker.wait().await;
    tracing::trace!("all tasks completed");

    Ok(())
}

/*
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


*/
