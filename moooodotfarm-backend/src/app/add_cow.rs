use crate::app::{CowTxtDownloader, Inventory, Metrics};
use crate::errors::{Error, Result};
use crate::{app, domain};
use anyhow::anyhow;
use async_trait::async_trait;

#[derive(Clone)]
pub struct AddCowHandler<I, D, M> {
    inventory: I,
    downloader: D,
    metrics: M,
}

impl<I, D, M> AddCowHandler<I, D, M> {
    pub fn new(inventory: I, downloader: D, metrics: M) -> Self {
        Self {
            inventory,
            downloader,
            metrics,
        }
    }
}

#[async_trait]
impl<I, D, M> app::AddCowHandler for AddCowHandler<I, D, M>
where
    I: Inventory + Send + Sync,
    D: CowTxtDownloader + Send + Sync,
    M: Metrics + Send + Sync,
{
    async fn add_cow(&self, v: &app::AddCow) -> Result<()> {
        self.downloader.download(v.name()).await?;

        self.inventory.update(v.name(), |status| {
            if status.is_some() {
                return Err(Error::Unknown(anyhow!("cow already exists")));
            }

            let cow = domain::Cow::new(v.name().clone(), v.character().clone());
            Ok(Some(cow))
        })?;
        Ok::<(), Error>(())
    }
}
