// TODO: CLEAN UP
use once_cell::sync::Lazy;
use std::borrow::Cow;
use uuid::{Uuid};
use poise::serenity_prelude::{self as serenity, CacheHttp};
use serenity::model::channel::AttachmentType;
use cdrs_tokio::{
  query_values,
  frame::TryFromRow,
  types::rows::Row
};
use crate::{
  util::{
    types::{Context, Error},
    error_handling::catch_unwind_silent,
  },
  database:: {
    schema::ImageData,
    cassandra,
    cassandra::CassandraConnection,
    CQL_PATH,
  }
};
use serenity::model::id::ChannelId;
static CQL_SHOW_VIA_ID: Lazy<String> = Lazy::new(|| std::fs::read_to_string(format!("{CQL_PATH}/SHOW_VIA_ID.sql")).expect("read `SHOW_VIA_ID.sql`"));
static CQL_FLAG_IMAGE: Lazy<String> = Lazy::new(|| std::fs::read_to_string(format!("{CQL_PATH}/FLAG_IMAGE.sql")).expect("read `SHOW_VIA_ID.sql`"));

/// Look up previously generated images.
#[poise::command(
  slash_command,
  category = "stablediffusion",
  rename = "report"
)]
pub async fn execute(
    ctx: Context<'_>,
    #[description = "The ID for the prompt to report"] id: String,
) -> Result<(), Error> {
  ctx.defer_ephemeral().await?;

  /* Command cooldowns */
  {
    let globals = ctx.data();
    let mut redis_pool = globals.redis_pool.get().await;

    let entry: String = format!("C_REPORT_{}", ctx.author().id);
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
  let image = get_image_data(database, CQL_SHOW_VIA_ID.clone(), Some(id.clone())).await;

  /* No image found matching `id` */
  if let None = image {
    ctx.say("<:cringeAkari:1059796702086844506> Could not find any images matching your given ID.").await?;

    return Ok(());
  }

  /* Flag image as reported */
  cassandra::query_update(database, CQL_FLAG_IMAGE.clone(), Some(query_values!(Uuid::parse_str(id.as_str().as_ref()).expect("uuid")))).await;

  /* Convert database entry to usable data */
  let image_parsed: ImageData = ImageData::try_from_row(image.unwrap()[0].to_owned()).expect("into RowStruct");

  if image_parsed.flagged {
    ctx.send(|m|
      m.content("<:cringeAkari:1059796702086844506> This image has already been report by other users, please be patient whilst we process the report.")
        .ephemeral(true)
    ).await?;

    return Ok(());
  }

  let grid_as_vec: Vec<u8> = image_parsed.grid_image.into_vec();
  /* Convert `grid_as_vec` to `AttachmentType` */
  let f: AttachmentType = AttachmentType::Bytes { data: Cow::from(&grid_as_vec), filename: "image.png".into() };

  /* Send message to reports channel for review */
  ChannelId(1059879830142849105).send_message(ctx.http(), |m|
    m.add_file(f)
      .embed(|e| {
        e.color(serenity::utils::Colour::from_rgb(47, 49, 54))
          .title(format!("{}", image_parsed.id))
          .description(format!("New report from ***{}*** (<@{}>)", ctx.author().tag(), ctx.author().id))
          .field("Creator (ID)", format!("<@{}>", image_parsed.origin_author), true)
          .footer(|f|
            f
              .text(format!("Requested by {}#{} | ko-fi.com/wessel", ctx.author().name, ctx.author().discriminator))
              .icon_url(match ctx.author().avatar_url() {
                Some(x) =>  x,
                None => ctx.author().default_avatar_url(),
              }))
          .image("attachment://image.png")
      })
  ).await?;

  ctx.send(|m|
    m.content("Your report has been submitted, thanks for your contribution!")
      .ephemeral(true)
  ).await?;

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

  if res.is_none() {
    None
  } else {
    Some(res.unwrap())
  }
  } else {
    let res = cassandra::query_search(&database, query.into(), None).await;
  if res.is_none() {
    None
  } else {
    Some(res.unwrap())
  }
  }
}