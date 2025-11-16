mod api;
mod controller;

use anyhow::Context as AnyhowContext;
use axum::{Router, http::StatusCode, routing::get};
use controller::{Context, TheLeagueController};
use kube::Client;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info,kube=trace")
        .init();
    info!("Starting TheLeague Controller (Idiomatic kube-rs).");

    let client = Client::try_default().await?;
    let context = Arc::new(Context {
        client: client.clone(),
    });

    // Equivalent to mgr.AddHealthzCheck("healthz", healthz.Ping) and mgr.AddReadyzCheck("readyz", healthz.Ping)
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz));

    // Default probe address (can be made configurable via env var like in Go)
    let probe_addr = std::env::var("PROBE_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let addr: SocketAddr = probe_addr
        .parse()
        .with_context(|| format!("Invalid probe address '{}'", probe_addr))?;

    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("Unable to bind health check server to {}", addr))?;
    info!("Health check server listening on {}", addr);

    let server = axum::serve(listener, app);

    info!("Starting reconciliation loop for TheLeague...");

    let league_controller = TheLeagueController::new(context.clone());
    let controller_stream = league_controller.stream();

    info!("Starting manager");
    tokio::select! {
        result = server => {
            if let Err(e) = result {
                error!(error = %e, "Problem running health check server");
                std::process::exit(1);
            } else {
                info!("Result: {:?}", result)
            }
        }
        _ = controller_stream => {
            info!("Controller stream ended");
        }
    }
    info!("Done!");
    Ok(())
}

// Health check endpoints (equivalent to healthz.Ping in Go)
async fn healthz() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok")
}

async fn readyz() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok")
}
