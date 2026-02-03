pub mod time;

use crate::domain::time::{DateTime, Duration};
use crate::errors::Error;
use crate::errors::Result;
use anyhow::anyhow;
use std::fmt::{Display, Formatter};

const COW_BODY: &str = include_str!("../ports/http/static/cow.txt");

const COW_SUFFIX: &str = "/cow.txt";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cow {
    url: url::Url,
}

impl Cow {
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let url = url::Url::parse(&s.into())?;
        if !url.path().ends_with(COW_SUFFIX) {
            return Err(Error::Unknown(anyhow!(
                "cow must have a tail and end with '{}'",
                COW_SUFFIX
            )));
        }
        Ok(Cow { url })
    }

    pub fn url(&self) -> &url::Url {
        &self.url
    }
}

impl Display for Cow {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}

pub fn cow_is_present(s: impl Into<String>) -> bool {
    let a: String = s.into();
    let a = a.trim();
    let b = COW_BODY.trim();
    let distance = edit_distance::edit_distance(a, b);
    distance < 100
}

#[derive(Debug, Clone)]
pub struct Herd {
    cows: Vec<Cow>,
}

impl Herd {
    pub fn new(mut cows: Vec<Cow>) -> Result<Self> {
        if cows.is_empty() {
            return Err(Error::Unknown(anyhow!("all the cows have escaped")));
        }
        cows.sort_by(|a, b| a.url().cmp(b.url()));

        for i in 1..cows.len() {
            if cows[i - 1].url() == cows[i].url() {
                return Err(Error::Unknown(anyhow!("duplicate cow found {}", cows[i])));
            }
        }

        Ok(Herd { cows })
    }

    pub fn cows(&self) -> &[Cow] {
        &self.cows
    }
}

#[derive(Clone)]
pub struct CowStatus {
    cow: Cow,
    first_seen: Option<DateTime>,
    last_seen: Option<DateTime>,
    last_checked: Option<DateTime>,
}

impl CowStatus {
    pub fn new(cow: Cow) -> Self {
        Self {
            cow,
            first_seen: None,
            last_seen: None,
            last_checked: None,
        }
    }

    pub fn new_from_history(
        cow: Cow,
        first_seen: Option<DateTime>,
        last_seen: Option<DateTime>,
        last_checked: Option<DateTime>,
    ) -> Self {
        Self {
            cow,
            first_seen,
            last_seen,
            last_checked,
        }
    }

    pub fn should_check(&self) -> bool {
        if let Some(last_checked) = &self.last_checked {
            let now = DateTime::now();
            return &now - last_checked > Duration::new_from_hours(2);
        }
        true
    }

    pub fn mark_as_ok(&mut self) {
        let now = DateTime::now();

        if self.first_seen.is_none() {
            self.first_seen = Some(now.clone());
        }

        self.last_seen = Some(now.clone());
        self.last_checked = Some(now.clone());
    }

    pub fn mark_as_missing(&mut self) {
        let now = DateTime::now();
        self.last_checked = Some(now.clone());
    }

    pub fn cow(&self) -> &Cow {
        &self.cow
    }

    pub fn first_seen(&self) -> Option<&DateTime> {
        self.first_seen.as_ref()
    }

    pub fn last_seen(&self) -> Option<&DateTime> {
        self.last_seen.as_ref()
    }

    pub fn last_checked(&self) -> Option<&DateTime> {
        self.last_checked.as_ref()
    }
}

pub trait Inventory {
    fn get(&self, cow: &Cow) -> Result<Option<CowStatus>>;
    fn put(&self, status: CowStatus) -> Result<()>;
}

#[derive(Clone)]
pub struct Rancher<I>
where
    I: Inventory,
{
    herd: Herd,
    inventory: I,
}

impl<I> Rancher<I>
where
    I: Inventory,
{
    pub fn new(herd: Herd, inventory: I) -> Self {
        Self { herd, inventory }
    }

    fn get_cow_status(&self, cow: &Cow) -> Result<CowStatus> {
        match self.inventory.get(cow)? {
            Some(cow_status) => Ok(cow_status),
            None => Ok(CowStatus::new(cow.clone())),
        }
    }

    async fn is_present(&self, cow: &Cow) -> Result<()> {
        let cow_body = reqwest::get(cow.url().to_string()).await?.text().await?;
        if !cow_is_present(&cow_body) {
            return Err(Error::Unknown(anyhow!("cow is not present: {}", cow_body)));
        }
        Ok(())
    }
}

impl<I> Rancher<I>
where
    I: Inventory,
{
    pub async fn update(&self) -> Result<()> {
        for cow in self.herd.cows() {
            let mut status = self.get_cow_status(cow)?;
            if !status.should_check() {
                continue;
            }

            match self.is_present(cow).await {
                Ok(_) => {
                    status.mark_as_ok();
                }
                Err(err) => {
                    log::warn!("cow is missing {}: {}", cow, err);
                    status.mark_as_missing();
                }
            }

            self.inventory.put(status)?;
        }
        Ok(())
    }

    pub fn get_cow_statuses(&self) -> Result<Vec<CowStatus>> {
        let mut statuses = vec![];
        for cow in self.herd.cows() {
            let status = self.get_cow_status(cow)?;
            statuses.push(status);
        }
        Ok(statuses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;
    use std::fs::read_to_string;

    #[test]
    fn cow_validation_works() -> Result<()> {
        let test_cow = read_to_string(fixtures::test_file_path("src/ports/http/static/cow.txt"))?;

        assert!(cow_is_present(&test_cow));
        assert!(!cow_is_present("not a cow"));
        Ok(())
    }

    #[test]
    fn duplicate_cows_in_herd_are_detected_even_if_they_are_not_next_to_each_other() {
        let cow1 = Cow::new("http://example.com/cow.txt").unwrap();
        let cow2 = Cow::new("http://example.org/cow.txt").unwrap();
        let cow3 = Cow::new("http://example.com/cow.txt").unwrap();

        let herd_result = Herd::new(vec![cow1, cow2, cow3]);
        assert!(herd_result.is_err());
    }
}
