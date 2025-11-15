use k8s_openapi::apimachinery::pkg::apis::meta::v1::Condition;
use kube::CustomResource;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// TheLeague is the Schema for the TheLeague API.
/// This defines the configuration and participating teams.
#[derive(CustomResource, Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[kube(
    group = "league.bexxmodd.com",
    version = "v1alpha1",
    kind = "TheLeague",
    plural = "theleagues",
    namespaced
)]
#[kube(status = "TheLeagueStatus")] // Status reports overall health, not standings
pub struct TheLeagueSpec {
    /// MaxTeams specifies the maximum number of teams allowed in the league (currently 8).
    #[serde(rename = "maxTeams")]
    #[schemars(
        validate(minimum=1),
        validate(maximum=8),
    )]
    pub max_teams: u8,

    /// Matchups defines the number of times any two teams must play each other.
    pub matchups: u32,

    /// Teams is the list of teams currently registered in the league.
    pub teams: Vec<Team>,
}

/// TheLeagueStatus defines the observed state of TheLeague.
#[derive(Deserialize, Serialize, Debug, Default, Clone, JsonSchema)]
pub struct TheLeagueStatus {
    /// Live indicates if the league is configured and the controller is running.
    pub live: bool,

    /// Conditions represent the latest available observations of an object's state.
    pub conditions: Option<Vec<Condition>>,
}

/// Team represents an individual team participating in the league.
#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
pub struct Team {
    /// Name is the unique identifier for the team.
    #[schemars(
        // Enforce alphanumeric characters and spaces only
        validate(pattern = "^[a-zA-Z0-9 ]+$") 
    )]
    pub name: String,

    /// Description provides an optional short description for the team.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Location is an optional field for the team's location or home field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// Players is the roster of players on this team.
    pub players: Vec<Player>,
}

/// Player represents an individual player on a team's roster.
#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
pub struct Player {
    /// FirstName is the first name of a player.
    #[serde(rename = "firstName")]
    #[schemars(
        validate(pattern = "^[a-zA-Z]+$")
    )]
    pub first_name: String,

    /// LastName is the last name of a player.
    #[serde(rename = "lastName")]
    #[schemars(
        validate(pattern = "^[a-zA-Z]+$")
    )]
    pub last_name: String,
}