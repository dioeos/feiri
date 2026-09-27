use crate::models::Mark;
use serde::{Deserialize, Serialize};

/// A query request asks for the current state within the Feiri daemon.
///
/// Query requests are sent to the daemon and perform operations such as asking for all
/// marks. They are sent by external applications, such as Feiri's `ui` crate.
#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QueryRequest {
    /// Lists all marks currently stored by the daemon.
    ListMarks,
}

/// A query response is the data corresponding to a particular query request that the 
/// Feiri daemon received. 
#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum QueryResponse {
    /// All the marks currently stored within the daemon.
    Marks(Vec<Mark>),
}
