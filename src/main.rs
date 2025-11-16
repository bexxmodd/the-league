mod api;

use anyhow::Context as AnyhowContext;
use api::v1alpha1::the_league_types::TheLeague;
use axum::{Router, http::StatusCode, routing::get};
use futures::StreamExt;
use kube::{
    Api, Client, ResourceExt,
    runtime::{
        controller::{Action, Controller},
        watcher,
    },
};
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, time::Duration};
use tracing::{error, info};

pub type Result<T, E = kube::Error> = std::result::Result<T, E>;

// --- Context and Reconciler Definition ---

/// Context shared between the controller and the worker threads
#[derive(Clone)]
struct Context {
    /// Kubernetes client
    _client: Client,
}

async fn reconcile(league: Arc<TheLeague>, _ctx: Arc<Context>) -> Result<Action, kube::Error> {
    info!("reconcile request: {}", league.name_any());
    Ok(Action::requeue(Duration::from_secs(3600)))
}

fn error_policy(_object: Arc<TheLeague>, _err: &kube::Error, _ctx: Arc<Context>) -> Action {
    info!("error policy: {}", _err);
    Action::requeue(Duration::from_secs(5))
}

// Health check endpoints (equivalent to healthz.Ping in Go)
async fn healthz() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok")
}

async fn readyz() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info,kube=trace")
        .init();
    info!("Starting TheLeague Controller (Idiomatic kube-rs).");

    let client = Client::try_default().await?;
    let context = Arc::new(Context {
        _client: client.clone(),
    });

    let league_api: Api<TheLeague> = Api::all(client.clone());

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
    let controller = Controller::new(league_api, watcher::Config::default())
        .shutdown_on_signal()
        .run(reconcile, error_policy, context)
        .for_each(|_| futures::future::ready(()));

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
        _ = controller => {
            info!("Controller stream ended");
        }
    }
    info!("Done!");
    Ok(())
}
