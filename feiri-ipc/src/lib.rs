#![allow(clippy::doc_lazy_continuation)]
//! Types for communicating with Feiri via IPC.
//!
//! After connecting to the Feiri socket, you can send [`Request`]s.
//!
//! ## Features
//!
//! This crate offers the following features:
//! - `clap`: derives the clap CLI parsing traits for the applicable traits. This
//! feature is used internally by the Feiri daemon to support CLI functionality.

use feiri_core::models::Mark;

/// Request from client to Feiri daemon
#[derive(serde::Deserialize, serde::Serialize)]
pub enum Request {
    /// Peform an operation
    Operation(Command),
    /// Receive events from the daemon continuously.
    EventStream,
}

/// Reply from Feiri daemon to client.
///
/// Every request receives on reply.
///
/// * If an error had occured during the daemon's handling of the request, the reply will be an `Reply::Err`.
/// * If the request does not need any particular response, the reply will be an `Reply::Ok(Response::Handled)`.
/// * Otherwise, it will be `Reply::Ok(response)` with one of the other [`Response`] variants.
pub type Reply = Result<Response, String>;

/// Successful response from Feiri to client
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum Response {
    Handled,
    Query(QueryResponse),
}

/// A type of request that performs a one-shot command to the daemon
#[derive(serde::Deserialize, serde::Serialize)]
pub enum Command {
    Action(Action),
    Query(Query),
}

/// An action that mutates or navigates Feiri state.
///
/// Action requests are sent to the daemon and perform operations such as assigning marks or focusing
/// marked windows. They are sent by external applications, such as Feiri's `cli` and `ui` crates.
#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[cfg_attr(feature = "clap", derive(clap::Subcommand))]
pub enum Action {
    /// Marks the currently focused window with the given slot.
    MarkWindow {
        /// Slot to assign to the focused window.
        ///
        /// For example: `1`.
        slot: u8,
    },

    /// Marks a specific window with the given slot.
    MarkRequestedWindow {
        /// Slot to assign to the window.
        slot: u8,

        /// Identifier of the window to mark.
        ///
        /// For example: `42`.
        #[cfg_attr(feature = "clap", arg(long))]
        id: u64,
    },

    /// Focuses the window assigned to the given mark slot.
    FocusMark {
        /// Slot of the mark to focus
        slot: u8,
    },

    /// Focuses the next marked window based on the last focused mark slot.
    ///
    /// For example, if the last focused mark slot was `1`, `NextMark` request would
    /// focus slot `2`, if present.
    NextMark,

    /// Focuses the previous marked window based on the last focused mark slot.
    ///
    /// For example, if the last focused mark slot was `2`, `PrevMark` request would
    /// focus slot `1`, if present.
    PrevMark,
}

/// A query request asks for the current state within the Feiri daemon.
///
/// Query requests are sent to the daemon and perform operations such as asking for all
/// marks. They are sent by external applications, such as Feiri's `ui` crate.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[cfg_attr(feature = "clap", derive(clap::Subcommand))]
pub enum Query {
    /// Lists all marks currently stored by the daemon.
    ListMarks,
}

/// A query response is the data corresponding to a particular query request that the
/// Feiri daemon received.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum QueryResponse {
    /// All the marks currently stored within the daemon.
    Marks(Vec<Mark>),
}

/// A daemon event
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum Event {
    /// The marks confiuguration has changed.
    MarksChanged {
        /// The new marks configuration.
        marks: Vec<Mark>,
    },
}

pub mod socket;
