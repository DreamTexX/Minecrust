use std::net::SocketAddr;

use tokio::net::TcpListener;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

mod connection;
mod dispatcher;

pub async fn run(
    cancellation_token: CancellationToken,
    tracker: TaskTracker,
    addr: SocketAddr,
) -> Result<(), tokio::io::Error> {
    let listener = TcpListener::bind(addr).await?;
    tracing::debug!(?addr, "listener created");

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
