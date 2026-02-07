pub mod add_cow;
pub mod change_cow_character;
pub mod check_cow;
pub mod get_herd;
pub mod update;

use crate::domain;
use crate::domain::Character;
use crate::domain::time::{DateTime, Duration};
use crate::errors::{Error, Result};

pub trait UpdateHandler {
    async fn handle(&self) -> Result<()>;
}

pub trait GetHerdHandler {
    fn get_herd(&self) -> Result<Herd>;
}

pub trait CheckCowHandler {
    async fn check_cow(&self, v: &CheckCow) -> Result<CheckCowResult<'_>>;
}

pub trait AddCowHandler {
    async fn add_cow(&self, v: &AddCow) -> Result<()>;
}

pub trait ChangeCowCharacterHandler {
    async fn change_cow_character(&self, v: &ChangeCowCharacter) -> Result<()>;
}

pub struct CheckCow {
    name: domain::VisibleName,
}

impl CheckCow {
    pub fn new(name: domain::VisibleName) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &domain::VisibleName {
        &self.name
    }
}

pub struct CheckCowResult<'a> {
    cow_txt: domain::CowTxt<'a>,
}

impl<'a> CheckCowResult<'a> {
    pub fn new(cow_txt: domain::CowTxt<'a>) -> Self {
        Self { cow_txt }
    }

    pub fn cow_txt(&self) -> &domain::CowTxt<'a> {
        &self.cow_txt
    }
}

pub struct AddCow {
    name: domain::VisibleName,
    character: Character,
}

impl AddCow {
    pub fn new(name: domain::VisibleName, character: Character) -> Self {
        Self { name, character }
    }

    pub fn name(&self) -> &domain::VisibleName {
        &self.name
    }

    pub fn character(&self) -> &Character {
        &self.character
    }
}

pub struct ChangeCowCharacter {
    name: domain::VisibleName,
    new_character: Character,
}

impl ChangeCowCharacter {
    pub fn new(name: domain::VisibleName, new_character: Character) -> Self {
        Self {
            name,
            new_character,
        }
    }

    pub fn name(&self) -> &domain::VisibleName {
        &self.name
    }

    pub fn character(&self) -> &Character {
        &self.new_character
    }
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

pub trait Inventory {
    fn get(&self, name: &domain::VisibleName) -> Result<Option<domain::Cow>>;
    fn list(&self) -> Result<Vec<domain::Cow>>;
    fn update<F>(&self, name: &domain::VisibleName, f: F) -> Result<()>
    where
        F: FnOnce(Option<domain::Cow>) -> Result<Option<domain::Cow>>;
}

pub trait CowTxtDownloader {
    async fn download(&self, name: &domain::VisibleName) -> Result<domain::CowTxt<'_>>;
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

impl TryFrom<Vec<domain::CensoredCow>> for Herd {
    type Error = Error;

    fn try_from(value: Vec<domain::CensoredCow>) -> Result<Self> {
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

impl TryFrom<&domain::CensoredCow> for Cow {
    type Error = Error;

    fn try_from(value: &domain::CensoredCow) -> Result<Self> {
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

    fn new(cow_status: &domain::CensoredCow) -> Self {
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
