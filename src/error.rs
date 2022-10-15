use std::io;

use base64::DecodeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LowestbinsError {
    #[error("{0}")]
    NbtError(#[from] nbt::Error),
    #[error("HTTP ERROR: {0}")]
    HttpError(#[from] isahc::Error),
    #[error("HTTP DECODE ERROR: {0}")]
    HttpDecodeError(#[from] isahc::http::Error),
    #[error("Error while parsing JSON: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[cfg(feature = "simd")]
    #[error("Error while parsing JSON: {0}")]
    SimdJsonError(#[from] simd_json::Error),
    #[error("Misc Error")]
    MiscError,
}

impl From<ctrlc::Error> for LowestbinsError {
    fn from(_: ctrlc::Error) -> Self {
        LowestbinsError::MiscError
    }
}

impl From<DecodeError> for LowestbinsError {
    fn from(_: DecodeError) -> Self {
        LowestbinsError::MiscError
    }
}

impl From<io::Error> for LowestbinsError {
    fn from(_: io::Error) -> Self {
        LowestbinsError::MiscError
    }
}

pub type Result<T> = std::result::Result<T, LowestbinsError>;
