use crate::app::{CowTxtDownloader, Inventory, Metrics};
use crate::errors::{Error, Result};
use crate::{app, domain};

#[derive(Clone)]
pub struct UpdateHandler<I, D, M> {
    inventory: I,
    downloader: D,
    metrics: M,
}

impl<I, D, M> UpdateHandler<I, D, M> {
    pub fn new(inventory: I, downloader: D, metrics: M) -> Self {
        Self {
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
    async fn handle(&self) -> Result<()> {
        let mut censored_statuses = vec![];

        for cow in self.inventory.list()? {
            if !cow.should_check() {
                continue;
            }

            let result = self.downloader.download(cow.name()).await;

            self.inventory.update(cow.name(), |status| {
                if let Some(mut status) = status {
                    match result {
                        Ok(_) => {
                            status.mark_as_ok();
                        }
                        Err(err) => {
                            log::warn!("cow is missing {}: {}", cow, err);
                            status.mark_as_missing();
                        }
                    }

                    let censored_status = domain::CensoredCow::new(&cow)?;
                    censored_statuses.push(censored_status);

                    return Ok(Some(status));
                }

                Ok(None)
            })?;
        }

        let herd: app::Herd = censored_statuses.try_into()?;
        self.metrics.update_herd_numbers(&herd);

        Ok::<(), Error>(())
    }
}
