use crate::app::UpdateHandler;
use log::{debug, error};
use std::time::Duration;
use tokio::time::sleep;

static UPDATE_EVERY: Duration = Duration::from_secs(60 * 5);

pub struct UpdateTimer<H: UpdateHandler> {
    handler: H,
}

impl<H> UpdateTimer<H>
where
    H: UpdateHandler,
{
    pub fn new(handler: H) -> Self {
        Self { handler }
    }

    pub async fn run(&self) {
        loop {
            match self.handler.handle().await {
                Ok(_) => {
                    debug!("executed update timer");
                }
                Err(err) => {
                    error!("error executing update timer: {}", err);
                }
            }
            sleep(UPDATE_EVERY).await;
        }
    }
}
