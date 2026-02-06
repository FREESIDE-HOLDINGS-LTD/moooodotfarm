pub mod get_herd;
pub mod update;

use crate::domain;
use crate::domain::time::{DateTime, Duration};
use crate::domain::{Character, VisibleName};
use crate::errors::{Error, Result};

pub trait UpdateHandler {
    async fn handle(&self) -> Result<()>;
}

pub trait GetHerdHandler {
    fn get_herd(&self) -> Result<Herd>;
}

pub trait Metrics {
    fn record_application_handler_call(
        &self,
        handler_name: &str,
        result: ApplicationHandlerCallResult,
        duration: Duration,
    );

    fn update_herd_numbers(&self, herd: &Herd);
}

pub enum ApplicationHandlerCallResult {
    Ok,
    Error,
}

pub struct Herd {
    cows: Vec<Cow>,
}

impl Herd {
    pub fn cows(&self) -> &[Cow] {
        &self.cows
    }
}

impl TryFrom<Vec<domain::CensoredCowStatus>> for Herd {
    type Error = Error;

    fn try_from(value: Vec<domain::CensoredCowStatus>) -> Result<Self> {
        let cows: Result<Vec<_>> = value.iter().map(Cow::try_from).collect();
        Ok(Self { cows: cows? })
    }
}

pub struct Cow {
    name: domain::Name,
    character: Character,
    last_seen: Option<DateTime>,
    status: CowStatus,
}

impl Cow {
    pub fn name(&self) -> &domain::Name {
        &self.name
    }

    pub fn character(&self) -> &Character {
        &self.character
    }

    pub fn last_seen(&self) -> Option<&DateTime> {
        self.last_seen.as_ref()
    }

    pub fn status(&self) -> &CowStatus {
        &self.status
    }
}

impl TryFrom<&domain::CensoredCowStatus> for Cow {
    type Error = Error;

    fn try_from(value: &domain::CensoredCowStatus) -> Result<Self> {
        Ok(Self {
            name: value.name().clone(),
            character: value.character().clone(),
            last_seen: value.last_seen().cloned(),
            status: CowStatus::new(value),
        })
    }
}

pub enum CowStatus {
    HappilyGrazing,
    RanAway,
    HaveNotCheckedYet,
}

impl CowStatus {
    pub fn all_variants() -> &'static [CowStatus] {
        &[
            CowStatus::HappilyGrazing,
            CowStatus::RanAway,
            CowStatus::HaveNotCheckedYet,
        ]
    }

    fn new(cow_status: &domain::CensoredCowStatus) -> Self {
        if cow_status.last_checked().is_none() {
            return CowStatus::HaveNotCheckedYet;
        }

        let seen_in_last_24h = cow_status
            .last_seen()
            .map(|v| DateTime::now() - v < Duration::new_from_hours(24))
            .unwrap_or(false);
        if seen_in_last_24h {
            return CowStatus::HappilyGrazing;
        }

        CowStatus::RanAway
    }
}

pub trait Inventory {
    fn get(&self, name: &VisibleName) -> Result<Option<domain::CowStatus>>;
    fn put(&self, status: domain::CowStatus) -> Result<()>;
}
