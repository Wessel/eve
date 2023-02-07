// TODO: CLEAN UP
use once_cell::sync::Lazy;
use std::borrow::Cow;
use uuid::{Uuid};
use serde_json::{from_str};
use poise::serenity_prelude::{self as serenity};
use serenity::model::channel::AttachmentType;
use cdrs_tokio::{
  query_values,
  frame::TryFromRow,
  types::rows::Row
};
use crate::{
  constants,
  util::{
    types::{Context, Error},
  },
  util::error_handling::catch_unwind_silent,
  database:: {
    schema::ImageData,
    cassandra,
    cassandra::CassandraConnection,
    CQL_PATH,
  }
};

static CQL_SHOW_VIA_ID: Lazy<String> = Lazy::new(|| std::fs::read_to_string(format!("{CQL_PATH}/SHOW_VIA_ID.sql")).expect("read `SHOW_VIA_ID.sql`"));
static CQL_SHOW_RANDOM: Lazy<String> = Lazy::new(|| std::fs::read_to_string(format!("{CQL_PATH}/SHOW_RANDOM.sql")).expect("read `SHOW_RANDOM.yml`"));

/// Look up previously generated images.
#[poise::command(
  slash_command,
  category = "stablediffusion",
  rename = "show"
)]
pub async fn execute(
    ctx: Context<'_>,
    #[description = "Show a random (leaving ID empty will set random to \"True\")"] mut random: Option<bool>,
    #[description = "The ID for the prompt to show (random if empty)"] id: Option<String>,
) -> Result<(), Error> {
  /* Command cooldowns */
  {
    let globals = ctx.data();
    let mut redis_pool = globals.redis_pool.get().await;

    let entry: String = format!("C_SHOW_{}", ctx.author().id);
    let now: i64 = chrono::Utc::now().timestamp();
    let key = redis_pool.get(&entry).await.unwrap();
     match key {
      Some(x) => {
        let u8_to_i64 = std::str::from_utf8(&x)
          .unwrap()
          .parse::<i64>()
          .unwrap();

        ctx.send(|m|m.content(format!("This command is on cooldown. (`{}s`)",  u8_to_i64 - now)).ephemeral(true)).await?;
   	    return Ok(());
      },
      None => {
        let cooldown_time: u32 = globals.config.cooldowns.show;
        if !globals.config.cooldowns._ignore.contains(ctx.author().id.as_u64()) {
          redis_pool.set_and_expire_ms(&entry, format!("{}", (now + (cooldown_time / 1000) as i64)), cooldown_time).await.unwrap();
        }
      }
    }
  }

  /* Argument parsing: id + random */
  let database: &CassandraConnection = &ctx.data().database;
  if let None = id { random = Some(true); }
  let image = match random {
    Some(true) => get_image_data(database, CQL_SHOW_RANDOM.clone(), None).await,
    Some(false) => get_image_data(database, CQL_SHOW_VIA_ID.clone(), Some(id.unwrap())).await,
    None => get_image_data(database, CQL_SHOW_VIA_ID.clone(), Some(id.unwrap())).await
  };

  /* No image found matching `id` */
  if let None = image {
    ctx.send(|m|
      m.content("<:cringeAkari:1059796702086844506> Could not find any images matching your given ID.")
        .ephemeral(true)
    ).await?;

    return Ok(());
  }

  /* Convert database entry to usable data */
  let image: ImageData = ImageData::try_from_row(image.unwrap()[0].to_owned()).expect("into RowStruct");
  let info: crate::structures::StableDiffusionConfig = from_str(image.settings.as_str().as_ref()).expect("settings as json");
  let grid_as_vec: Vec<u8> = image.grid_image.into_vec();
  /* Convert `grid_as_vec` to `AttachmentType` */
  let f: AttachmentType = AttachmentType::Bytes { data: Cow::from(&grid_as_vec), filename: "image.png".into() };
  /* Get `stablediffusion._notice_string` from `config.yml` */
  let notice_string: String = ctx.data()
    .config.stable_diffusion._notice_string
    .to_owned()
    .unwrap_or("".into());

  /* Send found image data */
  ctx.send(|m|
    m.attachment(f)
      .embed(|e| {
        e.color(serenity::utils::Colour::from_rgb(47, 49, 54))
          .title(format!("{}", image.id))
          .description(format!("{}\n\nImage generated in `{} seconds` (`{} steps`).\nCreated on <t:{}:F>", notice_string, image.job_time / 1000000000, &info.iterations, image.creation.timestamp()))
          .field("Seed", &info.seed, true)
          .field("Creator (ID)", format!("<@{}>", image.origin_author), true)
          .footer(|f|
            f
              .text(format!("Requested by {} | ko-fi.com/wessel | {}v{}", ctx.author().tag(), constants::NAME, constants::VERSION))
              .icon_url(match ctx.author().avatar_url() {
                Some(x) =>  x,
                None => ctx.author().default_avatar_url(),
              }))
          .image("attachment://image.png")
      }))
	  .await?;

  Ok(())
}

async fn get_image_data(database: &CassandraConnection, query: String, uuid: Option<String>) -> Option<Vec<Row>> {
  if uuid.is_some() {
    let result = catch_unwind_silent(||
      Uuid::parse_str(uuid.unwrap().as_str().as_ref()).expect("uuid")
    );

  let id = match result {
    Ok(x) => x,
    Err(_) => return None
  };

  let identifiers = query_values!(id);
  let res = cassandra::query_search(&database, query.into(), Some(identifiers)).await;
  Some(res.unwrap())
  } else {
    let res = cassandra::query_search(&database, query.into(), None).await;
    Some(res.unwrap())
  }
}