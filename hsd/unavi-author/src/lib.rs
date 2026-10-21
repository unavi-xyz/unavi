//! Undoable editing shared by every tool, and the history the shell holds.
//! `edit` is the tool side; `history` is the shell's.

#![expect(
    clippy::derive_partial_eq_without_eq,
    clippy::unsafe_derive_deserialize,
    reason = "the derives go on every wit-bindgen type, which this crate does not write"
)]

mod edit;
mod history;
mod protocol;
mod snapshot;

// The property types derive serde so a change can travel from a tool's
// instance of this library to the shell's as one message.
wit_bindgen::generate!({
    generate_all,
    with: {
        "wired:core/math": wired_guest::math,
    },
    additional_derives: [PartialEq, serde::Serialize, serde::Deserialize],
});

struct World;

impl exports::unavi::author::api::Guest for World {
    type Edit = edit::Recorder;
    type History = history::History;

    fn request_undo() {
        protocol::request(protocol::CH_UNDO);
    }

    fn request_redo() {
        protocol::request(protocol::CH_REDO);
    }
}

export!(World);
