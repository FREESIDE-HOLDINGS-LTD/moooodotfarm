use crate::app::{Inventory, Metrics};
use crate::domain::VisibleName;
use crate::errors::{Error, Result};
use crate::{app, domain};
use anyhow::anyhow;
use moooodotfarm_macros::application_handler;

#[derive(Clone)]
pub struct UpdateHandler<I, M> {
    herd: domain::Herd,
    inventory: I,
    metrics: M,
}

impl<I, M> UpdateHandler<I, M> {
    pub fn new(herd: domain::Herd, inventory: I, metrics: M) -> Self {
        Self {
            inventory,
            herd,
            metrics,
        }
    }
}

impl<I, M> app::UpdateHandler for UpdateHandler<I, M>
where
    I: Inventory,
    M: Metrics,
{
    #[application_handler]
    async fn handle(&self) -> Result<()> {
        for cow in self.herd.cows() {
            let mut status = self.get_or_create_cow_status(cow.name())?;
            if !status.should_check() {
                continue;
            }

            match self.is_present(cow).await {
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

impl<I, M> UpdateHandler<I, M>
where
    I: Inventory,
    M: Metrics,
{
    fn get_or_create_cow_status(&self, name: &VisibleName) -> Result<domain::CowStatus> {
        match self.inventory.get(name)? {
            Some(cow_status) => Ok(cow_status),
            None => Ok(domain::CowStatus::new(name.clone())),
        }
    }

    async fn is_present(&self, cow: &crate::domain::Cow) -> Result<()> {
        let cow_body = reqwest::get(cow.name().url().to_string())
            .await?
            .text()
            .await?;
        if !domain::cow_is_present(&cow_body) {
            return Err(Error::Unknown(anyhow!("cow is not present: {}", cow_body)));
        }
        Ok(())
    }
}
