use poise::serenity_prelude as serenity;

use crate::util::types::{Context, Error};
use poise::samples::HelpConfiguration;

// Get some general information about EVE and all registered commands.
#[poise::command(
  slash_command,
  prefix_command,
  rename = "help"
)]
pub async fn execute(
  ctx: Context<'_>,
  #[description = "get help for a specific command"]
  #[autocomplete = "poise::builtins::autocomplete_command"]
  command: Option<String>,) ->  Result<(), Error> {

    ctx.defer_ephemeral().await?;

  dbg!(&ctx.framework().options().commands);


  match command {
    Some(command) => {
      poise::builtins::help(
        ctx,
        Some(command.as_str()),
        HelpConfiguration {
        extra_text_at_bottom: format!("{} v{}", "EVE", "1.0.0")
        .as_str(),
        ..Default::default()
      }).await?;
    },
    None => {
        ctx.send(|m|
    m.embed(|e|
    e.description(r#"
    **`Notices`**<:ohRightAkari:709344398776729604>
    `(I)  ` This model is far from perfect. This model has been trained on an uncurated dataset of text-image pairs and thus some prompts may not be accurate.
    `(II) ` Images may be presented as black squares, this is due to the NSFW filter. This filter is set to be strict to avoid any accidental slip-ups from happening.
    `(III)` By using EVE, you accept that prompt-image pairs may be saved with their respective authors (in the form of ID) for furthur training/catalogue purposes.
    `(IV) ` Please note that, due to hardware restrictions, this bot is not able to run 24/7. If you would like to donate to support making the bot 24/7 in the future, please refer to my [Ko-Fi](https://ko-fi.com/wessel) (https://ko-fi.com/wessel).

    Invite EVE to your own server via [this link](https://discord.com/oauth2/authorize?client_id=609435148231901247&scope=bot&permissions=414464657472) (https://wessel.meek.moe/eve).

    As said above, any donations via [Ko-Fi](https://ko-fi.com/wessel) (https://ko-fi.com/wessel) will go into buying a sophisticated server in order to run 24/7.
    If you have any furthur questions, please refer to the "EVE" category in [this support server (Wessel's Duplicant Lab)](https://discord.gg/SV7DAE9) (https://discord.gg/SV7DAE9).
    "#)
    .field("Stable Diffusion", "`imagine`, `show`, `report`", true)
    .field("General", "`help`, `ping`", true)
    .color(serenity::utils::Colour::from_rgb(47, 49, 54)))
    .ephemeral(true)
  ).await?;
    }
  }

  Ok(())
}