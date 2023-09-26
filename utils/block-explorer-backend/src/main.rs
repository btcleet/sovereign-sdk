mod db;
mod indexer;
pub(crate) mod models;
mod routing;

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use clap::Parser;
use db::Db;
use sov_celestia_adapter::verifier::address::CelestiaAddress;
use sov_db::ledger_db::LedgerDB;
use sov_modules_stf_template::{SequencerOutcome, TxEffect};
use sov_rollup_interface::rpc::{LedgerRpcProvider, QueryMode, TxIdentifier};
use tracing::{info, trace, warn};

use crate::indexer::index_blocks;

type AppState = Arc<AppStateInner>;

#[derive(Clone)]
pub struct AppStateInner {
    db: Db,
    rpc: LedgerDB,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::parse();

    // Initialize the database.
    let db = Db::new(&config.db_connection_url).await?;
    // Connect to the RPC provider, which ironically in this case, is actually
    // the ledger RocksDB (so it's not really an RPC).
    // TODO: connect to the node via RPC.
    let rpc = LedgerDB::with_path(&config.ledger_db_path).expect("Failed to open ledger db");
    let app_state = Arc::new(AppStateInner { db, rpc });

    let app = Router::new().nest("/api/v0", routing::api_v0_router(app_state.clone()));
    let socket_addr: SocketAddr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, config.port).into();

    let app_state_clone = app_state.clone();
    tokio::task::spawn(index_blocks(
        app_state_clone,
        Duration::from_secs(config.polling_interval_in_secs),
    ));

    info!(socket_addr = socket_addr.to_string(), "Serving requests...");

    axum::Server::bind(&socket_addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(Debug, Parser)]
struct Config {
    #[clap(long, default_value = "2")]
    polling_interval_in_secs: u64,
    #[clap(long, env)]
    db_connection_url: String,
    #[clap(long, default_value = "3010")]
    port: u16,
    #[clap(long)]
    ledger_db_path: PathBuf,
}
