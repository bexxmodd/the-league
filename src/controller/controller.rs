use crate::api::v1alpha1::the_league_types::TheLeague;

use futures::StreamExt;
use kube::runtime::{controller::Controller as KubeController, watcher};
use kube::{Api, Client, ResourceExt, runtime::controller::Action};
use std::sync::Arc;
use tokio::time::Duration;
use tracing::info;

/// Context shared between the controller and the worker threads
#[derive(Clone)]
pub struct Context {
    /// Kubernetes client
    pub client: Client,
}

/// Controller for managing TheLeague resources
pub struct TheLeagueController {
    context: Arc<Context>,
    controller: KubeController<TheLeague>,
}

impl TheLeagueController {
    /// Create a new TheLeagueController
    pub fn new(context: Arc<Context>) -> Self {
        // Configure default namespace(s) - equivalent to cache.Options.DefaultNamespaces in Go
        // If WATCH_NAMESPACE is set, watch only that namespace; otherwise watch all namespaces
        let league_api: Api<TheLeague> = match std::env::var("WATCH_NAMESPACE") {
            Ok(namespace) if !namespace.is_empty() => {
                info!("Watching namespace: {}", namespace);
                Api::namespaced(context.client.clone(), &namespace)
            }
            _ => {
                info!("Watching all namespaces");
                Api::all(context.client.clone())
            }
        };

        // Configure watcher with cache options (equivalent to cache.Options in Go)
        // You can customize the watcher config here, e.g.:
        // - labels/field selectors
        // - backoff settings
        // - etc.
        let watcher_config = watcher::Config::default()
            // Example: Add label selector if needed
            // .labels("app=the-league")
            // Example: Custom backoff settings
            // .backoff(backoff::ExponentialBackoff::default())
            ;
        let controller = KubeController::new(league_api, watcher_config);
        Self {
            context,
            controller,
        }
    }

    /// Reconcile a TheLeague resource (static method)
    pub async fn reconcile(
        league: Arc<TheLeague>,
        _ctx: Arc<Context>,
    ) -> Result<Action, kube::Error> {
        info!("reconcile request: {}", league.name_any());
        Ok(Action::requeue(Duration::from_secs(3600)))
    }

    /// Handle errors that occur during reconciliation (static method)
    pub fn error_policy(_object: Arc<TheLeague>, err: &kube::Error, _ctx: Arc<Context>) -> Action {
        info!("error policy: {}", err);
        Action::requeue(Duration::from_secs(5))
    }

    pub fn stream(self) -> impl futures::Future<Output = ()> {
        let context = self.context.clone();
        self.controller
            .shutdown_on_signal()
            .run(
                TheLeagueController::reconcile,
                TheLeagueController::error_policy,
                context,
            )
            .for_each(|_| futures::future::ready(()))
    }
}
