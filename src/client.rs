use darkredis::{self, ConnectionPool};

// use log::{info};

use poise::serenity_prelude as serenity;
use crate::{commands, database};

use serde_yaml::{from_reader as parse_yaml};
use std::{
  fs::File,
};
use crate::util::types::{Error};
use crate::structures;
use crate::events;
use crate::structures::{GlobalData as Data};

pub async fn init(database_connection: database::cassandra::CassandraConnection, redis_pool: ConnectionPool) -> Result<poise::FrameworkBuilder<Data, Error>, Error> {
	/* Loading config file */
  let config_file = File::open("config.yml")
	.expect("Failed reading `config.yml`");

  let config: structures::Config = parse_yaml(config_file)
	  .expect("Failed parsing `config_file`");

  /* Creating Poise framework */
  let framework = poise::Framework::builder()
    .options(poise::FrameworkOptions {
      commands: commands::prepare(),
      event_handler: |ctx, event, _framework, data| Box::pin(events::handle(ctx, event, data)),
      ..Default::default()
    })
    .token(&config.authentication.discord)
    .intents(serenity::GatewayIntents::non_privileged())
    .setup(|ctx, _ready, framework| {
      Box::pin(async move {
        poise::builtins::register_globally(ctx, &framework.options().commands).await?;
        Ok(structures::GlobalData {
          config: config,
          redis_pool: redis_pool,
          start_time: std::time::Instant::now(),
          database: database_connection
        })
      })
    });

    Ok(framework)
}