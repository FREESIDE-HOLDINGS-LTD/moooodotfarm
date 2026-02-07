use crate::app;
use crate::app::{CheckCow, CheckCowResult, CowTxtDownloader, Metrics};
use crate::errors::{Error, Result};
use moooodotfarm_macros::application_handler;

#[derive(Clone)]
pub struct CheckCowHandler<D, M> {
    downloader: D,
    metrics: M,
}

impl<D, M> CheckCowHandler<D, M> {
    pub fn new(downloader: D, metrics: M) -> Self {
        Self {
            downloader,
            metrics,
        }
    }
}

impl<D, M> app::CheckCowHandler for CheckCowHandler<D, M>
where
    D: CowTxtDownloader,
    M: Metrics,
{
    #[application_handler]
    async fn check_cow(&self, v: CheckCow) -> Result<CheckCowResult<'_>> {
        let cow_txt = self.downloader.download(v.name()).await?;
        Ok::<CheckCowResult<'_>, Error>(CheckCowResult::new(cow_txt))
    }
}
