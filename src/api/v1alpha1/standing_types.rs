use k8s_openapi::apimachinery::pkg::apis::meta::v1::Condition;
use kube::CustomResource;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Standing is the Schema for the Standing API.
/// This resource tracks the calculated performance for a single Team.
#[derive(CustomResource, Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[kube(
    group = "bexxmodd.com",
    version = "v1alpha1",
    kind = "Standing",
    plural = "standings",
    namespaced
)]
#[kube(status = "StandingStatus")]
pub struct StandingSpec {
    /// LeagueName references the parent TheLeague resource this standing belongs to.
    #[serde(rename = "leagueName")]
    pub league_name: String,

    /// TeamName is the name of the team this standing corresponds to.
    #[serde(rename = "teamName")]
    pub team_name: String,

    /// Resolution defines the tie-breaking method used for calculating the standing.
    pub resolution: StandingResolution
}

/// StandingStatus defines the observed and computed state of the Standing.
/// This field is managed by the controller.
#[derive(Deserialize, Serialize, Debug, Default, Clone, JsonSchema)]
pub struct StandingStatus {
    /// Points is the total accumulated points for the team.
    pub points: u32,
    /// Wins is the total number of wins.
    pub wins: u32,
    /// Losses is the total number of losses.
    pub losses: u32,
    /// Draws is the total number of draws.
    pub draws: u32,

    /// Conditions represent the latest available observations of the Standing's state.
    pub conditions: Option<Vec<Condition>>,
}

/// StandingResolution defines the tie-breaking method used for the standings.
#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
pub enum StandingResolution {
    /// Head2Head resolution prioritizes the outcome of direct matches between tied teams.
    Head2Head,
    
    /// GoalDifference resolution prioritizes the overall goal difference across all matches.
    GoalDifference,
}