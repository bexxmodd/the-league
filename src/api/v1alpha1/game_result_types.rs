use kube::CustomResource;
use serde::{Deserialize, Serialize};

/// GameResult is the Schema for the GameResult API.
/// Each instance records the outcome of a single match.
#[derive(CustomResource, Deserialize, Serialize, Debug, Clone)]
#[kube(
    group = "league.bexxmodd.com",
    version = "v1alpha1",
    kind = "GameResult",
    plural = "gameresults",
    namespaced
)]
pub struct GameResultSpec {
    /// LeagueName references the parent TheLeague resource this game belongs to.
    pub league_name: String,

    /// RoundNumber indicates which round of the league schedule this game belongs to.
    pub round_number: u32,

    /// Teams contains the names of the two teams that played the game.
    pub teams: [Team; 2],

    /// Date is the time the game was played, preferably in RFC3339 format.
    pub date: String,

    /// Result specifies the outcome and scores of the game.
    pub result: GameOutcome,
}

/// GameOutcome defines the outcome and point distribution for the match.
/// (Winner: 3 points, Loser: 0 points, Draw: 1 point each)
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum GameOutcome {
    /// WinnerTeam0 indicates the first team in the `teams` array won.
    WinnerTeam0 { score_t0: u32, score_t1: u32 },
    /// WinnerTeam1 indicates the second team in the `teams` array won.
    WinnerTeam1 { score_t0: u32, score_t1: u32 },
    /// Draw indicates a tie game.
    Draw { score: u32 },
}