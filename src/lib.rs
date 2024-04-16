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

    #[error("nok response from meili: {code:03}. Body:{body:?}")]
    UnexpectedNok { code: u16, body: Option<String> },

    #[error("deserializing response: {err}. Body: \n{body}")]
    Deserialize {
        err: serde_json::Error,
        body: String,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
