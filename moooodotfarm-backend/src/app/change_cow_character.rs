use crate::app::{Inventory, Metrics};
use crate::errors::{Error, Result};
use crate::{app, domain};
use anyhow::anyhow;
use async_trait::async_trait;

#[derive(Clone)]
pub struct ChangeCowCharacterHandler<I, M> {
    inventory: I,
    metrics: M,
}

impl<I, M> ChangeCowCharacterHandler<I, M>
where
    I: Inventory,
    M: Metrics,
{
    pub fn new(inventory: I, metrics: M) -> Self {
        Self { inventory, metrics }
    }

    async fn change_cow_character_inner(&self, v: &app::ChangeCowCharacter) -> Result<()> {
        self.inventory
            .update(v.name(), |cow: Option<domain::Cow>| match cow {
                Some(mut cow) => {
                    cow.change_character(v.character().clone())?;
                    Ok(Some(cow))
                }
                None => Err(Error::Unknown(anyhow!("cow does not exist"))),
            })?;
        Ok::<(), Error>(())
    }
}

#[async_trait]
impl<I, M> app::ChangeCowCharacterHandler for ChangeCowCharacterHandler<I, M>
where
    I: Inventory + Send + Sync,
    M: Metrics + Send + Sync,
{
    async fn change_cow_character(&self, v: &app::ChangeCowCharacter) -> Result<()> {
        crate::record_application_handler_call!(
            self.metrics,
            "change_cow_character",
            self.change_cow_character_inner(v).await
        )
    }
}
