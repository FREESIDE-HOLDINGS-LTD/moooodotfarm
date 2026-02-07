use crate::domain::time::DateTime;
use crate::domain::{CowStatus, VisibleName};
use crate::errors::Result;
use crate::{app, domain};
use anyhow::Context;
use redb;
use redb::{ReadableDatabase, ReadableTable};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

const COW_STATUS_TABLE: redb::TableDefinition<String, String> =
    redb::TableDefinition::new("cow_status");

#[derive(Clone)]
pub struct Database {
    db: Arc<Mutex<redb::Database>>,
}

impl Database {
    pub fn new(path: impl Into<String>) -> Result<Self> {
        let db = redb::Database::create(path.into()).context("Failed to open database")?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
    }
}

impl app::Inventory for Database {
    fn get(&self, name: &VisibleName) -> Result<Option<CowStatus>> {
        let db = self.db.lock().unwrap();

        let read_txn = db.begin_read()?;
        match read_txn.open_table(COW_STATUS_TABLE) {
            Ok(table) => {
                let key = name.url().to_string();
                match table.get(key)? {
                    Some(v) => {
                        let persisted: PersistedCowStatus = serde_json::from_str(&v.value())?;
                        Ok(Some(persisted.try_into()?))
                    }
                    None => Ok(None),
                }
            }
            Err(e) => match e {
                redb::TableError::TableDoesNotExist(_a) => Ok(None),
                other => Err(other.into()),
            },
        }
    }

    fn update<F>(&self, name: &VisibleName, f: F) -> Result<()>
    where
        F: FnOnce(Option<domain::CowStatus>) -> Result<Option<domain::CowStatus>>,
    {
        let db = self.db.lock().unwrap();

        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(COW_STATUS_TABLE)?;
            let key = name.url().to_string();

            let cow_status: Option<domain::CowStatus> = match table.get(&key)? {
                Some(v) => {
                    let persisted: PersistedCowStatus = serde_json::from_str(&v.value())?;
                    Some(persisted.try_into()?)
                }
                None => None,
            };

            let new_cow_status = f(cow_status)?;

            if let Some(new_cow_status) = new_cow_status {
                let persisted: PersistedCowStatus = new_cow_status.into();
                let j = serde_json::to_string(&persisted)?;
                table.insert(key, j)?;
            }
        }
        Ok(write_txn.commit()?)
    }
}

#[derive(Serialize, Deserialize)]
pub struct PersistedCowStatus {
    cow: String,
    first_seen: Option<String>,
    last_seen: Option<String>,
    last_checked: Option<String>,
}

impl From<CowStatus> for PersistedCowStatus {
    fn from(value: CowStatus) -> Self {
        PersistedCowStatus {
            cow: value.name().into(),
            first_seen: value.first_seen().map(|dt| dt.into()),
            last_seen: value.last_seen().map(|dt| dt.into()),
            last_checked: value.last_checked().map(|dt| dt.into()),
        }
    }
}

impl TryInto<CowStatus> for PersistedCowStatus {
    type Error = crate::errors::Error;

    fn try_into(self) -> std::result::Result<CowStatus, Self::Error> {
        Ok(CowStatus::new_from_history(
            VisibleName::new(&self.cow)?,
            match self.first_seen {
                Some(dt_str) => Some(dt_str.try_into()?),
                None => None,
            },
            match self.last_seen {
                Some(dt_str) => Some(dt_str.try_into()?),
                None => None,
            },
            match self.last_checked {
                Some(dt_str) => Some(dt_str.try_into()?),
                None => None,
            },
        ))
    }
}

impl From<&VisibleName> for String {
    fn from(value: &VisibleName) -> Self {
        value.url().to_string()
    }
}

impl From<&domain::Character> for String {
    fn from(value: &domain::Character) -> Self {
        match value {
            domain::Character::Brave => "brave".to_string(),
            domain::Character::Shy => "shy".to_string(),
        }
    }
}

const DT_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";
impl From<&DateTime> for String {
    fn from(value: &DateTime) -> Self {
        value.format(DT_FORMAT)
    }
}

impl TryInto<DateTime> for String {
    type Error = crate::errors::Error;

    fn try_into(self) -> std::result::Result<DateTime, Self::Error> {
        DateTime::new_from_str(&self, DT_FORMAT)
    }
}
