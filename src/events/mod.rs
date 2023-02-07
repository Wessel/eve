use crate::util::types::{Error};
use crate::structures::GlobalData as Data;
use poise::{serenity_prelude::Context, Event};

mod on_ready;

pub async fn handle<'a>(ctx: &Context, event: &Event<'a>, _data: &Data) -> Result<(), Error> {
    match event {
        Event::Ready { .. } => on_ready::handle(ctx).await,
        _ => Ok(()),
    }
}