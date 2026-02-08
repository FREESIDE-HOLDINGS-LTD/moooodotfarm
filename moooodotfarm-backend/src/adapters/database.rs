use crate::domain::time::DateTime;
use crate::domain::{Character, Cow, VisibleName};
use crate::errors::Result;
use crate::{app, domain};
use anyhow::{Context, anyhow};
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
    fn get(&self, name: &VisibleName) -> Result<Option<domain::Cow>> {
        let db = self.db.lock().unwrap();

        let read_txn = db.begin_read()?;
        match read_txn.open_table(COW_STATUS_TABLE) {
            Ok(table) => {
                let key = name.url().to_string();
                match table.get(key)? {
                    Some(v) => {
                        let persisted: PersistedCow = serde_json::from_str(&v.value())?;
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

    fn list(&self) -> Result<Vec<Cow>> {
        let db = self.db.lock().unwrap();
        let read_txn = db.begin_read()?;
        let mut cows = Vec::new();
        match read_txn.open_table(COW_STATUS_TABLE) {
            Ok(table) => {
                for row in table.iter()? {
                    let (_key, value) = row?;
                    let persisted: PersistedCow = serde_json::from_str(&value.value())?;
                    cows.push(persisted.try_into()?);
                }
                Ok(cows)
            }
            Err(e) => match e {
                redb::TableError::TableDoesNotExist(_a) => Ok(Vec::new()),
                other => Err(other.into()),
            },
        }
    }

    fn update<F>(&self, name: &VisibleName, f: F) -> Result<()>
    where
        F: FnOnce(Option<domain::Cow>) -> Result<Option<domain::Cow>>,
    {
        let db = self.db.lock().unwrap();

        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(COW_STATUS_TABLE)?;
            let key = name.url().to_string();

            let cow_status: Option<domain::Cow> = match table.get(&key)? {
                Some(v) => {
                    let persisted: PersistedCow = serde_json::from_str(&v.value())?;
                    Some(persisted.try_into()?)
                }
                None => None,
            };

            let cow_to_save = f(cow_status)?;

            if let Some(cow_to_save) = cow_to_save {
                let persisted: PersistedCow = cow_to_save.into();
                let j = serde_json::to_string(&persisted)?;
                table.insert(key, j)?;
            }
        }
        Ok(write_txn.commit()?)
    }
}
#[derive(Serialize, Deserialize)]
pub struct PersistedCow {
    name: String,
    character: String,
    first_seen: Option<String>,
    last_seen: Option<String>,
    last_checked: Option<String>,
}

impl From<domain::Cow> for PersistedCow {
    fn from(value: domain::Cow) -> Self {
        PersistedCow {
            name: value.name().into(),
            character: value.character().into(),
            first_seen: value.first_seen().map(|dt| dt.into()),
            last_seen: value.last_seen().map(|dt| dt.into()),
            last_checked: value.last_checked().map(|dt| dt.into()),
        }
    }
}

impl TryInto<domain::Cow> for PersistedCow {
    type Error = crate::errors::Error;

    fn try_into(self) -> std::result::Result<domain::Cow, Self::Error> {
        Ok(domain::Cow::new_from_history(
            self.name.try_into()?,
            self.character.try_into()?,
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

impl TryInto<VisibleName> for String {
    type Error = crate::errors::Error;

    fn try_into(self) -> std::result::Result<VisibleName, Self::Error> {
        VisibleName::new(self)
    }
}

impl From<&Character> for String {
    fn from(value: &domain::Character) -> Self {
        match value {
            domain::Character::Brave => "brave".to_string(),
            domain::Character::Shy => "shy".to_string(),
        }
    }
}

impl TryFrom<String> for Character {
    type Error = crate::errors::Error;

    fn try_from(value: String) -> std::result::Result<Character, Self::Error> {
        match value.as_str() {
            "brave" => Ok(Character::Brave),
            "shy" => Ok(Character::Shy),
            other => Err(Self::Error::Unknown(anyhow!(
                "unknown character: {}",
                other
            ))),
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
