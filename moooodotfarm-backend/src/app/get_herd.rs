use crate::app;
use crate::app::Herd;
use crate::errors::{Error, Result};
use moooodotfarm_macros::application_handler;

#[derive(Clone)]
pub struct GetHerdHandler<R, M> {
    rancher: R,
    metrics: M,
}

impl<R, M> GetHerdHandler<R, M> {
    pub fn new(rancher: R, metrics: M) -> Self {
        Self { rancher, metrics }
    }
}

impl<R, M> app::GetHerdHandler for GetHerdHandler<R, M>
where
    M: app::Metrics,
    R: app::Rancher,
{
    #[application_handler]
    fn get_herd(&self) -> Result<Herd> {
        let herd: Herd = self.rancher.get_cow_statuses()?.try_into()?;
        Ok::<Herd, Error>(herd)
    }
}
