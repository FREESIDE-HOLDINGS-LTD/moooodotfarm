use crate::app::{CowTxtDownloader, Inventory, Metrics};
use crate::domain::CensoredHerd;
use crate::errors::{Error, Result};
use crate::{app, domain};
use async_trait::async_trait;

macro_rules! record_application_handler_call {
    ($metrics:expr, $handler_name:expr, $expr:expr) => {{
        let start = crate::domain::time::DateTime::now();
        let result = $expr;
        $metrics.record_application_handler_call(
            $handler_name,
            (&result).into(),
            &crate::domain::time::DateTime::now() - &start,
        );
        result
    }};
}

#[derive(Clone)]
pub struct UpdateHandler<I, D, M> {
    inventory: I,
    downloader: D,
    metrics: M,
}

impl<I, D, M> UpdateHandler<I, D, M>
where
    I: Inventory + Send + Sync,
    D: CowTxtDownloader + Send + Sync,
    M: Metrics + Send + Sync,
{
    pub fn new(inventory: I, downloader: D, metrics: M) -> Self {
        Self {
            inventory,
            downloader,
            metrics,
        }
    }

    async fn handle_inner(&self) -> Result<()> {
        let mut cows: Vec<domain::Cow> = vec![];

        for peeked_cow in self.inventory.list()? {
            if !peeked_cow.should_check() {
                continue;
            }

            let result = self.downloader.download(peeked_cow.name()).await;

            self.inventory.update(peeked_cow.name(), |cow| {
                if let Some(mut cow) = cow {
                    match result {
                        Ok(_) => {
                            cow.mark_as_ok();
                        }
                        Err(err) => {
                            log::warn!("cow is missing {}: {}", cow, err);
                            cow.mark_as_missing();
                        }
                    }

                    cows.push(cow.clone());

                    return Ok(Some(cow));
                }

                Ok(None)
            })?;
        }

        let censored_cows: Vec<domain::CensoredCow> =
            cows.iter()
                .map(domain::CensoredCow::new)
                .collect::<Result<Vec<domain::CensoredCow>>>()?;
        let censored_herd = CensoredHerd::new(censored_cows);
        let herd: app::Herd = censored_herd.try_into()?;
        self.metrics.update_herd_numbers(&herd);

        Ok::<(), Error>(())
    }
}

#[async_trait]
impl<I, D, M> app::UpdateHandler for UpdateHandler<I, D, M>
where
    I: Inventory + Send + Sync,
    D: CowTxtDownloader + Send + Sync,
    M: Metrics + Send + Sync,
{
    async fn handle(&self) -> Result<()> {
        record_application_handler_call!(self.metrics, "update", self.handle_inner().await)
    }
}
