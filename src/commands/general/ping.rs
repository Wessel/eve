use poise::serenity_prelude as serenity;
use tokio::time::Instant;

use crate::util::types::{Context, Error};

use serenity::ShardId;

#[poise::command(
  slash_command,
  prefix_command,
  rename = "ping"
)]
pub async fn execute(ctx: Context<'_>) ->  Result<(), Error> {
    let now = Instant::now();
    reqwest::get("https://discordapp.com/api/v6/gateway").await?;
    let get_latency = now.elapsed().as_millis();

    let shard_latency = {
        let shard_manager = ctx.framework().shard_manager();
        let manager = shard_manager.lock().await;
        let runners = manager.runners.lock().await;

        let runner_raw = runners.get(&ShardId(ctx.serenity_context().shard_id));
        dbg!(&runners, ctx.serenity_context().shard_id, runner_raw);
        if let Some(runner) = runner_raw {
            match runner.latency {
                Some(ms) => format!("{}ms", ms.as_millis()),
                _ => "?ms".to_string(),
            }
        } else {
            "?ms".to_string()
        }
    };


    let now = Instant::now();
    let message = ctx.say("Calculating...").await?;
    let post_latency = now.elapsed().as_millis();

    message
        .edit(ctx, |m| {
            m.content("");
            m.embed(|e| {
                e.title("Latency");
                e.description(format!(
                    "Gateway: {}\nREST GET: {}ms\nREST POST: {}ms",
                    shard_latency, get_latency, post_latency
                ))
            })
        })
        .await?;

  Ok(())
}