mod client;
mod has_index;
mod index;
mod search;
mod task;

pub use client::Client;
pub use has_index::*;
pub use index::*;
pub use search::*;
pub use task::*;

pub mod prelude {
    pub use super::{HasIndex, HasIndexExt};
}

pub type DateTime = String;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("transport: {0}")]
    Transport(#[from] reqwest::Error),

    #[error("nok response from meili: {0:03}")]
    UnexpectedNok(u16),

    #[error("deserializing response: {err}. Body: \n{body}")]
    Deserialize {
        err: serde_json::Error,
        body: String,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
