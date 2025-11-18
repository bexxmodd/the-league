use crate::api::v1alpha1::the_league_types::{TheLeague, TheLeagueStatus};

use futures::StreamExt;
use k8s_openapi::apimachinery::pkg::apis::meta::v1;
use k8s_openapi::chrono;
use kube::runtime::{controller::Controller as KubeController, watcher};
use kube::{Api, Client, ResourceExt, runtime::controller::Action};
use kube::api;
use std::sync::Arc;
use tokio::time::Duration;
use tracing::{info, error};

/// Context shared between the controller and the worker threads
#[derive(Clone)]
pub struct Context {
    /// Kubernetes client
    pub client: Client,
}

/// Controller for managing TheLeague resources
pub struct Reconciler {
    context: Arc<Context>,
    controller: KubeController<TheLeague>,
}

impl Reconciler {
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
        ctx: Arc<Context>,
    ) -> Result<Action, kube::Error> {
        info!("reconcile request: {}", league.name_any());
        let name = league.name_any();
        let namespace = league.namespace().unwrap_or_default();
        let client = ctx.client.clone();
        let league_api: Api<TheLeague> = Api::namespaced(client, &namespace);

        let league = match league_api.get(&name).await {
            Ok(resource) => {
                info!("TheLeague '{}' found. Proceeding with reconciliation.", name);
                resource
            }
            Err(kube::Error::Api(e)) if e.code == 404 => {
                info!("TheLeague resource not found (404). Ignoring since object must be deleted.");
                return Ok(Action::await_change()); 
            }
            Err(e) => {
                // Error reading the object - requeue the request.
                error!("Failed to get TheLeague: {:?}", e);
                return Err(e)
            }
        };
        let current_conditions = league.status.as_ref().map(|s| &s.conditions).unwrap_or(&vec![]);
        if !current_conditions.is_empty() {
            // 1. Define initial status condition
            let initial_condition = v1::Condition {
                type_: String::from("Processing"),
                status: "Unknown".to_string(), // Equivalent to metav1.ConditionUnknown
                reason: String::from("Reconciling"),
                message: "Starting reconciliation".to_string(),
                // Required timestamp and generation fields
                last_transition_time:v1::Time(chrono::Utc::now()),
                observed_generation: league.metadata.generation, 
            };

            // 2. Create the initial status object for patching
            let initial_status = TheLeagueStatus {
                live: false, 
                conditions: vec![initial_condition],
            };

            //     // 3. Patch Status: Equivalent to Go's `r.Status().Update()`
            // let status_patch = api::Patch::Merge(TheLeague {
            //     status: Some(initial_status),
            //     // Ensure other fields are defaulted/ignored during the status patch
            //     ..TheLeague::new(&name, )
            // });
        }

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
            .run(Reconciler::reconcile, Reconciler::error_policy, context)
            .for_each(|_| futures::future::ready(()))
    }
}
