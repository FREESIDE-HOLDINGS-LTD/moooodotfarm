use anyhow::anyhow;
use clap::parser::MatchesError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("cow is not present in `{0}`")]
    CowIsNotPresent(String),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Unknown(anyhow!(value))
    }
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Error::Unknown(anyhow!(value))
    }
}

impl From<prometheus::Error> for Error {
    fn from(value: prometheus::Error) -> Self {
        Error::Unknown(anyhow!(value))
    }
}

impl From<chrono::ParseError> for Error {
    fn from(value: chrono::ParseError) -> Self {
        Error::Unknown(anyhow!(value))
    }
}

impl From<chrono::RoundingError> for Error {
    fn from(value: chrono::RoundingError) -> Self {
        Error::Unknown(anyhow!(value))
    }
}

impl From<url::ParseError> for Error {
    fn from(value: url::ParseError) -> Self {
        Error::Unknown(anyhow!(value))
    }
}

impl From<MatchesError> for Error {
    fn from(value: MatchesError) -> Self {
        Error::Unknown(anyhow!(value))
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::Unknown(anyhow!(value))
    }
}

impl From<redb::DatabaseError> for Error {
    fn from(value: redb::DatabaseError) -> Self {
        Error::Unknown(anyhow!(value))
    }
}

impl From<redb::CommitError> for Error {
    fn from(value: redb::CommitError) -> Self {
        Error::Unknown(anyhow!(value))
    }
}

impl From<redb::StorageError> for Error {
    fn from(value: redb::StorageError) -> Self {
        Error::Unknown(anyhow!(value))
    }
}

impl From<redb::TableError> for Error {
    fn from(value: redb::TableError) -> Self {
        Error::Unknown(anyhow!(value))
    }
}

impl From<redb::TransactionError> for Error {
    fn from(value: redb::TransactionError) -> Self {
        Error::Unknown(anyhow!(value))
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Unknown(anyhow!(value))
    }
}

impl From<tonic::Status> for Error {
    fn from(value: tonic::Status) -> Self {
        Error::Unknown(anyhow!(value))
    }
}

impl From<std::env::VarError> for Error {
    fn from(value: std::env::VarError) -> Self {
        Error::Unknown(anyhow!(value))
    }
}

impl From<tonic::transport::Error> for Error {
    fn from(value: tonic::transport::Error) -> Self {
        Error::Unknown(anyhow!(value))
    }
}

pub type Result<T> = std::result::Result<T, Error>;
