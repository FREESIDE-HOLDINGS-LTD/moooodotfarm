use crate::app::{Herd, Inventory, Metrics};
use crate::domain::CensoredHerd;
use crate::errors::Result;
use crate::{app, domain};
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
        let cows = self.inventory.list()?;
        let censored_cows = cows
            .into_iter()
            .map(|cow| domain::CensoredCow::new(&cow))
            .collect::<Result<Vec<domain::CensoredCow>>>()?;
        CensoredHerd::new(censored_cows).try_into()
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
