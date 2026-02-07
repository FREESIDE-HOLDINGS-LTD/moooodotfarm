use crate::errors::Result;
use anyhow::anyhow;

#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    http_address: String,
    grpc_address: String,
    environment: Environment,
    database_path: String,
}

impl Config {
    pub fn new(
        http_address: impl Into<String>,
        grpc_address: impl Into<String>,
        environment: Environment,
        database_path: impl Into<String>,
    ) -> Result<Self> {
        let http_address = http_address.into();
        if http_address.is_empty() {
            return Err(anyhow!("http_address can't be empty").into());
        }
        let grpc_address = grpc_address.into();
        if grpc_address.is_empty() {
            return Err(anyhow!("grpc_address can't be empty").into());
        }
        let database_path = database_path.into();
        if database_path.is_empty() {
            return Err(anyhow!("database_path can't be empty").into());
        }
        Ok(Self {
            http_address,
            grpc_address,
            environment,
            database_path,
        })
    }

    pub fn http_address(&self) -> &str {
        &self.http_address
    }

    pub fn grpc_address(&self) -> &str {
        &self.grpc_address
    }

    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    pub fn database_path(&self) -> &str {
        &self.database_path
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Environment {
    Production,
    Development,
}
