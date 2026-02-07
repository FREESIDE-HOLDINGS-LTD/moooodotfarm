use crate::app;
use crate::app::{Inventory, Metrics};
use crate::errors::{Error, Result};
use anyhow::anyhow;
use moooodotfarm_macros::application_handler;

#[derive(Clone)]
pub struct ChangeCowCharacterHandler<I, M> {
    inventory: I,
    metrics: M,
}

impl<I, M> ChangeCowCharacterHandler<I, M> {
    pub fn new(inventory: I, metrics: M) -> Self {
        Self { inventory, metrics }
    }
}

impl<I, M> app::ChangeCowCharacterHandler for ChangeCowCharacterHandler<I, M>
where
    I: Inventory,
    M: Metrics,
{
    #[application_handler]
    async fn change_cow_character(&self, v: &app::ChangeCowCharacter) -> Result<()> {
        self.inventory.update(v.name(), |cow| match cow {
            Some(mut cow) => {
                cow.change_character(v.character().clone())?;
                Ok(Some(cow))
            }
            None => Err(Error::Unknown(anyhow!("cow does not exist"))),
        })?;
        Ok::<(), Error>(())
    }
}
