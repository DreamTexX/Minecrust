use std::net::SocketAddr;

use clap::{Arg, ArgAction, Command, command, value_parser};
use tokio::signal;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let matches = command!()
        .about("Command line utility for Minecrust servers.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .arg(
            Arg::new("endpoint")
                .long("endpoint")
                .global(true)
                .help("endpoint(s) for etcd cluster")
                .value_parser(value_parser!(SocketAddr))
                .action(ArgAction::Append),
        )
        .subcommand(
            Command::new("gateway").about("Runs the gateway.").arg(
                Arg::new("addr")
                    .value_parser(value_parser!(SocketAddr))
                    .default_value("127.0.0.1:25565"),
            ),
        )
        .get_matches();

    let task_tracker = TaskTracker::new();
    let cancellation_token = CancellationToken::new();

    match matches.subcommand() {
        Some(("gateway", matches)) => {
            let addr = matches
                .get_one::<SocketAddr>("addr")
                .expect("addr is required");

            tracing::info!(?addr, "starting gateway");

            let gateway_handle = task_tracker.spawn(minecrust_gateway::run(
                cancellation_token.clone(),
                task_tracker.clone(),
                *addr,
            ));

            tokio::select! {
                biased;
                _ = signal::ctrl_c() => {
                    tracing::debug!("aborting because of signal");
                },
                result = gateway_handle => {
                    match result {
                        Ok(Err(err)) => {
                            tracing::error!(?err, "starting gateway failed");
                        },
                        Err(err) => {
                            tracing::error!(?err, "starting gateway failed");
                        }
                        _ => {}
                    }
                }
            }
        }
        _ => unreachable!(),
    }

    tracing::info!("shutting down, please wait");
    cancellation_token.cancel();
    task_tracker.close();
    task_tracker.wait().await;
}
