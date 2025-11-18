//! Binary to generate RBAC YAML files for the TheLeague controller.
//!
//! This follows kube.rs security best practices:
//! - https://kube.rs/controllers/security/#access-constriction
//! - Least-privilege principle
//! - Proper RBAC declarations
//!
//! Run with: `cargo run --bin generate-rbac`

use k8s_openapi::api::core::v1::ServiceAccount;
use k8s_openapi::api::rbac::v1::{ClusterRole, ClusterRoleBinding, PolicyRule, RoleRef, Subject};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use serde_yaml;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const GROUP: &str = "bexxmodd.com";
const SERVICE_ACCOUNT_NAME: &str = "theleague-controller-manager";
const ROLE_NAME: &str = "manager-role";
const LEADER_ELECTION_ROLE_NAME: &str = "leader-election-role";
const ADMIN_ROLE_NAME: &str = "theleague-admin-role";
const EDITOR_ROLE_NAME: &str = "theleague-editor-role";
const VIEWER_ROLE_NAME: &str = "theleague-viewer-role";
const APP_NAME: &str = "theleague";

/// Generate the main ClusterRole with permissions for CRDs
///
/// Following kube.rs security guidelines:
/// - ClusterRole is used because the controller can watch all namespaces
/// - Least-privilege: only the exact verbs needed for each resource
/// - Status subresources are explicitly scoped
fn generate_manager_role() -> ClusterRole {
    ClusterRole {
        metadata: ObjectMeta {
            name: Some(ROLE_NAME.to_string()),
            ..Default::default()
        },
        rules: Some(vec![
            // TheLeague CRD permissions
            PolicyRule {
                api_groups: Some(vec![GROUP.to_string()]),
                resources: Some(vec!["theleagues".to_string()]),
                verbs: vec![
                    "get".to_string(),
                    "list".to_string(),
                    "watch".to_string(),
                    "create".to_string(),
                    "update".to_string(),
                    "patch".to_string(),
                    "delete".to_string(),
                ],
                ..Default::default()
            },
            // TheLeague status permissions
            PolicyRule {
                api_groups: Some(vec![GROUP.to_string()]),
                resources: Some(vec!["theleagues/status".to_string()]),
                verbs: vec!["get".to_string(), "update".to_string(), "patch".to_string()],
                ..Default::default()
            },
            // Standing CRD permissions
            PolicyRule {
                api_groups: Some(vec![GROUP.to_string()]),
                resources: Some(vec!["standings".to_string()]),
                verbs: vec![
                    "get".to_string(),
                    "list".to_string(),
                    "watch".to_string(),
                    "create".to_string(),
                    "update".to_string(),
                    "patch".to_string(),
                    "delete".to_string(),
                ],
                ..Default::default()
            },
            // Standing status permissions
            PolicyRule {
                api_groups: Some(vec![GROUP.to_string()]),
                resources: Some(vec!["standings/status".to_string()]),
                verbs: vec!["get".to_string(), "update".to_string(), "patch".to_string()],
                ..Default::default()
            },
            // GameResult CRD permissions
            PolicyRule {
                api_groups: Some(vec![GROUP.to_string()]),
                resources: Some(vec!["gameresults".to_string()]),
                verbs: vec![
                    "get".to_string(),
                    "list".to_string(),
                    "watch".to_string(),
                    "create".to_string(),
                    "update".to_string(),
                    "patch".to_string(),
                    "delete".to_string(),
                ],
                ..Default::default()
            },
            // Events permissions (for controller events)
            PolicyRule {
                api_groups: Some(vec!["".to_string()]),
                resources: Some(vec!["events".to_string()]),
                verbs: vec!["create".to_string(), "patch".to_string()],
                ..Default::default()
            },
        ]),
        ..Default::default()
    }
}

/// Generate leader election ClusterRole
///
/// Required for controller coordination when multiple replicas run.
/// Uses coordination.k8s.io/leases for leader election.
fn generate_leader_election_role() -> ClusterRole {
    ClusterRole {
        metadata: ObjectMeta {
            name: Some(LEADER_ELECTION_ROLE_NAME.to_string()),
            ..Default::default()
        },
        rules: Some(vec![PolicyRule {
            api_groups: Some(vec!["coordination.k8s.io".to_string()]),
            resources: Some(vec!["leases".to_string()]),
            verbs: vec![
                "get".to_string(),
                "list".to_string(),
                "watch".to_string(),
                "create".to_string(),
                "update".to_string(),
                "patch".to_string(),
                "delete".to_string(),
            ],
            ..Default::default()
        }]),
        ..Default::default()
    }
}

/// Generate ServiceAccount
///
/// The ServiceAccount that the controller pods will use.
/// Namespace can be specified via NAMESPACE environment variable.
fn generate_service_account(namespace: Option<&str>) -> ServiceAccount {
    ServiceAccount {
        metadata: ObjectMeta {
            name: Some(SERVICE_ACCOUNT_NAME.to_string()),
            namespace: namespace.map(|s| s.to_string()),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Generate ClusterRoleBinding for the manager role
fn generate_role_binding(namespace: Option<&str>) -> ClusterRoleBinding {
    ClusterRoleBinding {
        metadata: ObjectMeta {
            name: Some(ROLE_NAME.to_string()),
            ..Default::default()
        },
        role_ref: RoleRef {
            api_group: "rbac.authorization.k8s.io".to_string(),
            kind: "ClusterRole".to_string(),
            name: ROLE_NAME.to_string(),
        },
        subjects: Some(vec![Subject {
            kind: "ServiceAccount".to_string(),
            name: SERVICE_ACCOUNT_NAME.to_string(),
            namespace: namespace.map(|s| s.to_string()),
            ..Default::default()
        }]),
        ..Default::default()
    }
}

/// Generate ClusterRoleBinding for leader election
fn generate_leader_election_role_binding(namespace: Option<&str>) -> ClusterRoleBinding {
    ClusterRoleBinding {
        metadata: ObjectMeta {
            name: Some(LEADER_ELECTION_ROLE_NAME.to_string()),
            ..Default::default()
        },
        role_ref: RoleRef {
            api_group: "rbac.authorization.k8s.io".to_string(),
            kind: "ClusterRole".to_string(),
            name: LEADER_ELECTION_ROLE_NAME.to_string(),
        },
        subjects: Some(vec![Subject {
            kind: "ServiceAccount".to_string(),
            name: SERVICE_ACCOUNT_NAME.to_string(),
            namespace: namespace.map(|s| s.to_string()),
            ..Default::default()
        }]),
        ..Default::default()
    }
}

/// Generate admin ClusterRole
///
/// This rule is not used by the project theleague itself.
/// It is provided to allow the cluster admin to help manage permissions for users.
///
/// Grants full permissions ('*') over bexxmodd.com resources.
/// This role is intended for users authorized to modify roles and bindings within the cluster,
/// enabling them to delegate specific permissions to other users or groups as needed.
fn generate_admin_role() -> ClusterRole {
    ClusterRole {
        metadata: ObjectMeta {
            name: Some(ADMIN_ROLE_NAME.to_string()),
            labels: Some({
                let mut labels = BTreeMap::new();
                labels.insert("app.kubernetes.io/name".to_string(), APP_NAME.to_string());
                labels.insert(
                    "app.kubernetes.io/managed-by".to_string(),
                    "kustomize".to_string(),
                );
                labels
            }),
            ..Default::default()
        },
        rules: Some(vec![
            // TheLeague full permissions
            PolicyRule {
                api_groups: Some(vec![GROUP.to_string()]),
                resources: Some(vec!["theleagues".to_string()]),
                verbs: vec!["*".to_string()],
                ..Default::default()
            },
            // TheLeague status permissions
            PolicyRule {
                api_groups: Some(vec![GROUP.to_string()]),
                resources: Some(vec!["theleagues/status".to_string()]),
                verbs: vec!["get".to_string()],
                ..Default::default()
            },
            // Standing full permissions
            PolicyRule {
                api_groups: Some(vec![GROUP.to_string()]),
                resources: Some(vec!["standings".to_string()]),
                verbs: vec!["*".to_string()],
                ..Default::default()
            },
            // Standing status permissions
            PolicyRule {
                api_groups: Some(vec![GROUP.to_string()]),
                resources: Some(vec!["standings/status".to_string()]),
                verbs: vec!["get".to_string()],
                ..Default::default()
            },
            // GameResult full permissions
            PolicyRule {
                api_groups: Some(vec![GROUP.to_string()]),
                resources: Some(vec!["gameresults".to_string()]),
                verbs: vec!["*".to_string()],
                ..Default::default()
            },
        ]),
        ..Default::default()
    }
}

/// Generate editor ClusterRole
///
/// This rule is not used by the project theleague itself.
/// It is provided to allow the cluster admin to help manage permissions for users.
///
/// Grants permissions to create, update, and delete resources within the bexxmodd.com.
/// This role is intended for users who need to manage these resources
/// but should not control RBAC or manage permissions for others.
fn generate_editor_role() -> ClusterRole {
    ClusterRole {
        metadata: ObjectMeta {
            name: Some(EDITOR_ROLE_NAME.to_string()),
            labels: Some({
                let mut labels = BTreeMap::new();
                labels.insert("app.kubernetes.io/name".to_string(), APP_NAME.to_string());
                labels.insert(
                    "app.kubernetes.io/managed-by".to_string(),
                    "kustomize".to_string(),
                );
                labels
            }),
            ..Default::default()
        },
        rules: Some(vec![
            // TheLeague editor permissions
            PolicyRule {
                api_groups: Some(vec![GROUP.to_string()]),
                resources: Some(vec!["theleagues".to_string()]),
                verbs: vec![
                    "create".to_string(),
                    "delete".to_string(),
                    "get".to_string(),
                    "list".to_string(),
                    "patch".to_string(),
                    "update".to_string(),
                    "watch".to_string(),
                ],
                ..Default::default()
            },
            // TheLeague status permissions
            PolicyRule {
                api_groups: Some(vec![GROUP.to_string()]),
                resources: Some(vec!["theleagues/status".to_string()]),
                verbs: vec!["get".to_string()],
                ..Default::default()
            },
            // Standing editor permissions
            PolicyRule {
                api_groups: Some(vec![GROUP.to_string()]),
                resources: Some(vec!["standings".to_string()]),
                verbs: vec![
                    "create".to_string(),
                    "delete".to_string(),
                    "get".to_string(),
                    "list".to_string(),
                    "patch".to_string(),
                    "update".to_string(),
                    "watch".to_string(),
                ],
                ..Default::default()
            },
            // Standing status permissions
            PolicyRule {
                api_groups: Some(vec![GROUP.to_string()]),
                resources: Some(vec!["standings/status".to_string()]),
                verbs: vec!["get".to_string()],
                ..Default::default()
            },
            // GameResult editor permissions
            PolicyRule {
                api_groups: Some(vec![GROUP.to_string()]),
                resources: Some(vec!["gameresults".to_string()]),
                verbs: vec![
                    "create".to_string(),
                    "delete".to_string(),
                    "get".to_string(),
                    "list".to_string(),
                    "patch".to_string(),
                    "update".to_string(),
                    "watch".to_string(),
                ],
                ..Default::default()
            },
        ]),
        ..Default::default()
    }
}

/// Generate viewer ClusterRole
///
/// This rule is not used by the project theleague itself.
/// It is provided to allow the cluster admin to help manage permissions for users.
///
/// Grants read-only access to bexxmodd.com resources.
/// This role is intended for users who need visibility into these resources
/// without permissions to modify them. It is ideal for monitoring purposes and limited-access viewing.
fn generate_viewer_role() -> ClusterRole {
    ClusterRole {
        metadata: ObjectMeta {
            name: Some(VIEWER_ROLE_NAME.to_string()),
            labels: Some({
                let mut labels = BTreeMap::new();
                labels.insert("app.kubernetes.io/name".to_string(), APP_NAME.to_string());
                labels.insert(
                    "app.kubernetes.io/managed-by".to_string(),
                    "kustomize".to_string(),
                );
                labels
            }),
            ..Default::default()
        },
        rules: Some(vec![
            // TheLeague viewer permissions
            PolicyRule {
                api_groups: Some(vec![GROUP.to_string()]),
                resources: Some(vec!["theleagues".to_string()]),
                verbs: vec!["get".to_string(), "list".to_string(), "watch".to_string()],
                ..Default::default()
            },
            // TheLeague status permissions
            PolicyRule {
                api_groups: Some(vec![GROUP.to_string()]),
                resources: Some(vec!["theleagues/status".to_string()]),
                verbs: vec!["get".to_string()],
                ..Default::default()
            },
            // Standing viewer permissions
            PolicyRule {
                api_groups: Some(vec![GROUP.to_string()]),
                resources: Some(vec!["standings".to_string()]),
                verbs: vec!["get".to_string(), "list".to_string(), "watch".to_string()],
                ..Default::default()
            },
            // Standing status permissions
            PolicyRule {
                api_groups: Some(vec![GROUP.to_string()]),
                resources: Some(vec!["standings/status".to_string()]),
                verbs: vec!["get".to_string()],
                ..Default::default()
            },
            // GameResult viewer permissions
            PolicyRule {
                api_groups: Some(vec![GROUP.to_string()]),
                resources: Some(vec!["gameresults".to_string()]),
                verbs: vec!["get".to_string(), "list".to_string(), "watch".to_string()],
                ..Default::default()
            },
        ]),
        ..Default::default()
    }
}

/// Write a Kubernetes resource to a YAML file
fn write_resource<T: serde::Serialize>(
    resource: &T,
    filename: &str,
    output_dir: &Path,
) -> anyhow::Result<()> {
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    let yaml = serde_yaml::to_string(resource)?;
    let file_path = output_dir.join(filename);
    fs::write(&file_path, yaml)?;
    Ok(())
}

/// Generate all RBAC manifests
///
/// Generates:
/// - ClusterRole with CRD permissions
/// - ClusterRole for leader election
/// - ServiceAccount
/// - ClusterRoleBindings
fn generate_all_rbac(output_dir: &Path, namespace: Option<&str>) -> anyhow::Result<()> {
    // Generate ClusterRole
    let role = generate_manager_role();
    write_resource(&role, "role.yaml", output_dir)?;
    println!("✓ Generated {}/role.yaml", output_dir.display());

    // Generate leader election ClusterRole
    let leader_role = generate_leader_election_role();
    write_resource(&leader_role, "leader_election_role.yaml", output_dir)?;
    println!(
        "✓ Generated {}/leader_election_role.yaml",
        output_dir.display()
    );

    // Generate ServiceAccount
    let sa = generate_service_account(namespace);
    write_resource(&sa, "service_account.yaml", output_dir)?;
    println!("✓ Generated {}/service_account.yaml", output_dir.display());

    // Generate ClusterRoleBinding
    let binding = generate_role_binding(namespace);
    write_resource(&binding, "role_binding.yaml", output_dir)?;
    println!("✓ Generated {}/role_binding.yaml", output_dir.display());

    // Generate leader election ClusterRoleBinding
    let leader_binding = generate_leader_election_role_binding(namespace);
    write_resource(
        &leader_binding,
        "leader_election_role_binding.yaml",
        output_dir,
    )?;
    println!(
        "✓ Generated {}/leader_election_role_binding.yaml",
        output_dir.display()
    );

    // Generate admin role (for cluster admins to delegate permissions)
    let admin_role = generate_admin_role();
    write_resource(&admin_role, "theleague_admin_role.yaml", output_dir)?;
    println!(
        "✓ Generated {}/theleague_admin_role.yaml",
        output_dir.display()
    );

    // Generate editor role (for cluster admins to delegate permissions)
    let editor_role = generate_editor_role();
    write_resource(&editor_role, "theleague_editor_role.yaml", output_dir)?;
    println!(
        "✓ Generated {}/theleague_editor_role.yaml",
        output_dir.display()
    );

    // Generate viewer role (for cluster admins to delegate permissions)
    let viewer_role = generate_viewer_role();
    write_resource(&viewer_role, "theleague_viewer_role.yaml", output_dir)?;
    println!(
        "✓ Generated {}/theleague_viewer_role.yaml",
        output_dir.display()
    );

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let output_dir = Path::new("config/rbac");

    // Get namespace from environment or use default
    // Following kube.rs best practice: deploy controller to its own namespace
    let namespace = std::env::var("NAMESPACE").ok();

    generate_all_rbac(output_dir, namespace.as_deref())?;

    println!("\nAll RBAC manifests generated successfully!");
    println!("Apply them with: kubectl apply -k config/rbac/");
    println!("\nNote: These manifests follow kube.rs security best practices:");
    println!("  - Least-privilege principle");
    println!("  - ClusterRole used because controller can watch all namespaces");
    println!("  - Explicit status subresource permissions");
    println!("  - Leader election permissions for controller coordination");

    Ok(())
}
