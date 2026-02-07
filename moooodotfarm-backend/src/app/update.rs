use crate::app::{CowTxtDownloader, Inventory, Metrics};
use crate::domain::VisibleName;
use crate::errors::{Error, Result};
use crate::{app, domain};
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
        for cow in self.herd.cows() {
            let mut status = self.get_or_create_cow_status(cow.name())?;
            if !status.should_check() {
                continue;
            }

            match self.downloader.download(cow.name()).await {
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

        // let herd: Herd = self.rancher.get_cow_statuses()?.try_into()?;
        // self.metrics.update_herd_numbers(&herd);

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
