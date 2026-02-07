use crate::app::{CowTxtDownloader, Inventory, Metrics};
use crate::errors::{Error, Result};
use crate::{app, domain};
use anyhow::anyhow;
use moooodotfarm_macros::application_handler;

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

impl<I, D, M> app::AddCowHandler for AddCowHandler<I, D, M>
where
    I: Inventory,
    D: CowTxtDownloader,
    M: Metrics,
{
    #[application_handler]
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
