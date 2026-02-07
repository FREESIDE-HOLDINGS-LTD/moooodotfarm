use crate::app::{CowTxtDownloader, Inventory, Metrics};
use crate::domain::VisibleName;
use crate::errors::{Error, Result};
use crate::{app, domain};
use anyhow::anyhow;
use moooodotfarm_macros::application_handler;

#[derive(Clone)]
pub struct UpdateHandler<I, D, M> {
    herd: domain::Herd,
    inventory: I,
    downloader: D,
    metrics: M,
}

impl<I, D, M> UpdateHandler<I, D, M> {
    pub fn new(herd: domain::Herd, inventory: I, downloader: D, metrics: M) -> Self {
        Self {
            herd,
            inventory,
            downloader,
            metrics,
        }
    }
}

impl<I, D, M> app::UpdateHandler for UpdateHandler<I, D, M>
where
    I: Inventory,
    D: CowTxtDownloader,
    M: Metrics,
{
    #[application_handler]
    async fn handle(&self) -> Result<()> {
        let mut censored_statuses = vec![];

        for cow in self.herd.cows() {
            let status = self.get_or_create_cow_status(cow.name())?;
            if !status.should_check() {
                continue;
            }

            let result = self.downloader.download(cow.name()).await;

            self.inventory.update(cow.name(), |status| {
                let mut status =
                    status.ok_or_else(|| anyhow!("cow status not found for {}", cow))?;
                match result {
                    Ok(_) => {
                        status.mark_as_ok();
                    }
                    Err(err) => {
                        log::warn!("cow is missing {}: {}", cow, err);
                        status.mark_as_missing();
                    }
                }

                let censored_status = domain::CensoredCowStatus::new(cow, &status)?;
                censored_statuses.push(censored_status);

                Ok(Some(status))
            })?;
        }

        let herd: app::Herd = censored_statuses.try_into()?;
        self.metrics.update_herd_numbers(&herd);

        Ok::<(), Error>(())
    }
}

impl<I, D, M> UpdateHandler<I, D, M>
where
    I: Inventory,
    D: CowTxtDownloader,
    M: Metrics,
{
    fn get_or_create_cow_status(&self, name: &VisibleName) -> Result<domain::CowStatus> {
        match self.inventory.get(name)? {
            Some(cow_status) => Ok(cow_status),
            None => Ok(domain::CowStatus::new(name.clone())),
        }
    }
}
