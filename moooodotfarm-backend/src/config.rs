use crate::domain::Cow;
use crate::errors::Result;
use anyhow::anyhow;

#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    address: String,
    environment: Environment,
    database_path: String,
    cows: Vec<Cow>,
}

impl Config {
    pub fn new(
        address: impl Into<String>,
        environment: Environment,
        database_path: impl Into<String>,
        cows: Vec<Cow>,
    ) -> Result<Self> {
        let address = address.into();
        if address.is_empty() {
            return Err(anyhow!("address can't be empty").into());
        }
        let database_path = database_path.into();
        if database_path.is_empty() {
            return Err(anyhow!("database_path can't be empty").into());
        }
        if cows.is_empty() {
            return Err(anyhow!("cows can't be empty").into());
        }
        Ok(Self {
            address,
            environment,
            database_path,
            cows,
        })
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    pub fn database_path(&self) -> &str {
        &self.database_path
    }

    pub fn cows(&self) -> &[Cow] {
        &self.cows
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Environment {
    Production,
    Development,
}
