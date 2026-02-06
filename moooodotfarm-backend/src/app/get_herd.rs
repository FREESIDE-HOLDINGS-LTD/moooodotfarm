use crate::app::{Herd, Inventory, Metrics};
use crate::domain::{CensoredCowStatus, VisibleName};
use crate::errors::{Error, Result};
use crate::{app, domain};
use moooodotfarm_macros::application_handler;

#[derive(Clone)]
pub struct GetHerdHandler<I, M> {
    herd: domain::Herd,
    inventory: I,
    metrics: M,
}

impl<I, M> GetHerdHandler<I, M> {
    pub fn new(herd: domain::Herd, inventory: I, metrics: M) -> Self {
        Self {
            herd,
            inventory,
            metrics,
        }
    }
}

impl<I, M> app::GetHerdHandler for GetHerdHandler<I, M>
where
    I: Inventory,
    M: Metrics,
{
    #[application_handler]
    fn get_herd(&self) -> Result<Herd> {
        let mut statuses = vec![];
        for cow in self.herd.cows() {
            let status = self.get_or_create_cow_status(cow.name())?;
            let censored_status = CensoredCowStatus::new(cow, &status)?;
            statuses.push(censored_status);
        }

        let herd: Herd = statuses.try_into()?;
        Ok::<Herd, Error>(herd)
    }
}

impl<I, M> GetHerdHandler<I, M>
where
    I: Inventory,
    M: Metrics,
{
    fn get_or_create_cow_status(&self, name: &VisibleName) -> Result<domain::CowStatus> {
        match self.inventory.get(name)? {
            Some(cow_status) => Ok(cow_status),
            None => Ok(domain::CowStatus::new(name.clone())),
        }
    }
}
