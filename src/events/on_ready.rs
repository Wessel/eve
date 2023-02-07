use poise::serenity_prelude as serenity;
use serenity::{
    model::{gateway::Activity},
    Context
};
use crate::util::types::{Error};

pub(crate) async fn handle(ctx: &Context) -> Result<(), Error> {
    log::info!("bot ready");
    ctx.set_activity(Activity::listening("/help")).await;

    Ok(())
}