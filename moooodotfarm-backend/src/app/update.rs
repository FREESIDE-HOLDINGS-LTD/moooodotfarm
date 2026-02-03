use crate::app;
use crate::app::{Herd, Metrics, Rancher};
use crate::errors::{Error, Result};
use moooodotfarm_macros::application_handler;

#[derive(Clone)]
pub struct UpdateHandler<R, M> {
    rancher: R,
    metrics: M,
}

impl<R, M> UpdateHandler<R, M> {
    pub fn new(rancher: R, metrics: M) -> Self {
        Self { rancher, metrics }
    }
}

impl<R, M> app::UpdateHandler for UpdateHandler<R, M>
where
    R: Rancher,
    M: Metrics,
{
    #[application_handler]
    async fn handle(&self) -> Result<()> {
        self.rancher.update().await?;

        let herd: Herd = self.rancher.get_cow_statuses()?.try_into()?;
        self.metrics.update_herd_numbers(&herd);

        Ok::<(), Error>(())
    }
}
