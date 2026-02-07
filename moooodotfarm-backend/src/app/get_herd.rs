use crate::app::{Herd, Inventory, Metrics};
use crate::errors::{Error, Result};
use crate::{app, domain};
use moooodotfarm_macros::application_handler;

#[derive(Clone)]
pub struct GetHerdHandler<I, M> {
    inventory: I,
    metrics: M,
}

impl<I, M> GetHerdHandler<I, M> {
    pub fn new(inventory: I, metrics: M) -> Self {
        Self { inventory, metrics }
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
        for cow in self.inventory.list()? {
            let censored_status = domain::CensoredCow::new(&cow)?;
            statuses.push(censored_status);
        }
        let herd: Herd = statuses.try_into()?;
        Ok::<Herd, Error>(herd)
    }
}
