use crate::app::{Inventory, Metrics};
use crate::errors::Result;
use crate::app;
use async_trait::async_trait;

#[derive(Clone)]
pub struct DeleteCowHandler<I, M> {
    inventory: I,
    metrics: M,
}

impl<I, M> DeleteCowHandler<I, M>
where
    I: Inventory,
    M: Metrics,
{
    pub fn new(inventory: I, metrics: M) -> Self {
        Self { inventory, metrics }
    }

    async fn handle_inner(&self, v: &app::DeleteCow) -> Result<()> {
        self.inventory.delete(v.name())?;
        Ok(())
    }
}

#[async_trait]
impl<I, M> app::DeleteCowHandler for DeleteCowHandler<I, M>
where
    I: Inventory + Send + Sync,
    M: Metrics + Send + Sync,
{
    async fn handle(&self, v: &app::DeleteCow) -> Result<()> {
        crate::record_application_handler_call!(
            self.metrics,
            "delete_cow",
            self.handle_inner(v).await
        )
    }
}

