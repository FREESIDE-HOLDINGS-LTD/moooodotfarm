use crate::app::{CowTxtDownloader, Inventory, Metrics};
use crate::errors::{Error, Result};
use crate::{app, domain};
use moooodotfarm_macros::application_handler;

#[derive(Clone)]
pub struct AddCowHandler<I, D, M> {
    herd: domain::Herd,
    inventory: I,
    downloader: D,
    metrics: M,
}

impl<I, D, M> AddCowHandler<I, D, M> {
    pub fn new(herd: domain::Herd, inventory: I, downloader: D, metrics: M) -> Self {
        Self {
            herd,
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
    async fn add_cow(&self, _v: &app::AddCow) -> Result<()> {
        Ok::<(), Error>(())
    }
}
