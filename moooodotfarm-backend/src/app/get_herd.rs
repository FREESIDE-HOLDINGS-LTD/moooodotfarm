use crate::app;
use crate::app::{Herd, Inventory, Metrics};
use crate::errors::{Error, Result};
use async_trait::async_trait;

#[derive(Clone)]
pub struct GetHerdHandler<I, M> {
    inventory: I,
    metrics: M,
}

impl<I, M> GetHerdHandler<I, M>
where
    I: Inventory,
    M: Metrics,
{
    pub fn new(inventory: I, metrics: M) -> Self {
        Self { inventory, metrics }
    }

    async fn handle_inner(&self) -> Result<Herd> {
        let mut statuses = vec![];
        for cow in self.inventory.list()? {
            let censored_status = crate::domain::CensoredCow::new(&cow)?;
            statuses.push(censored_status);
        }
        let herd: Herd = statuses.try_into()?;
        Ok::<Herd, Error>(herd)
    }
}

#[async_trait]
impl<I, M> app::GetHerdHandler for GetHerdHandler<I, M>
where
    I: Inventory + Send + Sync,
    M: Metrics + Send + Sync,
{
    async fn handle(&self) -> Result<Herd> {
        crate::record_application_handler_call!(self.metrics, "get_herd", self.handle_inner().await)
    }
}
