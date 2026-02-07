use crate::app;
use crate::app::{Inventory, Metrics};
use crate::errors::{Error, Result};
use anyhow::anyhow;
use async_trait::async_trait;

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

#[async_trait]
impl<I, M> app::ChangeCowCharacterHandler for ChangeCowCharacterHandler<I, M>
where
    I: Inventory + Send + Sync,
    M: Metrics + Send + Sync,
{
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
