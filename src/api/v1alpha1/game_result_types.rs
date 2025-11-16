use kube::CustomResource;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::Time;

/// GameResult is the Schema for the GameResult API.
/// Each instance records the outcome of a single match.
#[derive(CustomResource, Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[kube(
    group = "bexxmodd.com",
    version = "v1alpha1",
    kind = "GameResult",
    plural = "gameresults",
    namespaced
)]
pub struct GameResultSpec {
    /// LeagueName references the parent TheLeague resource this game belongs to.
    #[serde(rename = "leagueName")]
    pub league_name: String,

    /// RoundNumber indicates which round of the league schedule this game belongs to.
    #[serde(rename = "roundNumber")]
    pub round_number: u32,

    /// Teams contains the names of the two teams that played the game.
    pub teams: [String; 2],

    /// Time is the time the game was played, preferably in RFC3339 format.
    pub time: Time,

    /// Result specifies the outcome and scores of the game.
    pub result: GameOutcome,
}

/// GameOutcome defines the outcome and point distribution for the match.
/// (Winner: 3 points, Loser: 0 points, Draw: 1 point each)
#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
pub enum GameOutcome {
    /// WinnerHomeTeam indicates the team whose name is the FIRST element 
    /// in the `teams` array won (the 'Home' team).
    WinnerHomeTeam { score_home: u32, score_away: u32 },
    
    /// WinnerAwayTeam indicates the team whose name is the SECOND element 
    /// in the `teams` array won (the 'Away' team).
    WinnerAwayTeam { score_home: u32, score_away: u32 },
    
    /// Draw indicates a tie game.
    Draw { score: u32 },
}