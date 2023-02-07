// TODO: CLEAN UP
// TODO: UPLOAD CAP TO CASSANDRA DB

use std::{
  fs::File,
  borrow::Cow,
  io::Cursor,
//    panic::catch_unwind,
};

use once_cell::sync::Lazy;

use serenity::{
    model::channel::AttachmentType,
};

use serde_json;
use serde_yaml::{from_reader as parse_yaml};

use uuid::Uuid;
use base64;
use image as imagec;
use chrono::{Utc};
use reqwest::{
    Client as webClient,
    header::{CONTENT_TYPE}
};

use crate::structures::{
    StableDiffusionResponse,
    StableDiffusionConfig
};

use crate::{database::cassandra, database::schema::ImageData, database::CQL_PATH};
use crate::constants;
use crate::util::string::{replace_key_value, parse_cli_args, ellipsis};
use crate::util::types::{Context, Error};


static CQL_INSERT_IMAGE: Lazy<String> = Lazy::new(|| std::fs::read_to_string(format!("{CQL_PATH}/INSERT_IMAGE.sql")).expect("read `SHOW_VIA_ID.sql`"));

static STYLES: Lazy<serde_json::Value> = Lazy::new(|| {
	let styles_file = File::open("styles.yml")
		.expect("Failed reading `styles.yml`");

    parse_yaml(styles_file)
	    .expect("Failed parsing `styles_file`")
});


use poise::serenity_prelude as serenity;

/// Make EVE turn your prompt into a set of images.
#[poise::command(slash_command, prefix_command, category = "stablediffusion", rename = "imagine")]
pub async fn execute(
    ctx: Context<'_>,
    #[description = "The input prompt to use"] prompt: String,
    #[description = "The negative prompt to use (parts that you do not want in your end result)"] negative: Option<String>,
    #[description = "The style to use"] style: Option<String>,
    #[description = "The seed to use"] seed: Option<i64>,
    #[description = "The CFG scale to use (how close the prompt should be)"]
    #[rename = "prompt_distance"]
    cfg_scale: Option<f64>,
    #[description = "Whether or not to use face restoration"]
    #[rename = "face_restoration"]
    restore_faces: Option<bool>,
    #[description = "Which size to use for the image (square, portrait or landscape)"] size: Option<String>,
    #[description = "Use fast mode (lesser steps, faster processing)"] fast: Option<bool>,
) -> Result<(), Error> {
  let globals = ctx.data();
  let database = &ctx.data().database;

  /* Command cooldowns */
  {
    let mut conn = ctx.data().redis_pool.get().await;
    let entry = format!("C_IMAGINE_{}", ctx.author().id);
    let now = Utc::now().timestamp();
    let key = conn.get(&entry).await.unwrap();
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
        let cooldown_time = globals.config.cooldowns.imagine;
        if !globals.config.cooldowns._ignore.contains(ctx.author().id.as_u64()) {
          conn.set_and_expire_ms(&entry, format!("{}", (now + (cooldown_time / 1000) as i64)), cooldown_time).await.unwrap();
        }
      }
    }
  }

  let start_time = Utc::now();

    let fuuid = Uuid::new_v4();

  let original_message = ctx.send(|m| m.content(format!("`[{}]` generating, this may take a while...", fuuid)).ephemeral(true)).await?;

  let yaml_string = std::fs::read_to_string("subsitutions.yml")
    .expect("Failed to read YAML file");

  /* Argument parsing: prompt, negative */
  let split_args: Vec<&str> = prompt.split("||").collect();
  let prompt = replace_key_value(split_args[0].to_owned(), yaml_string);
  let negative = if split_args.len() > 1  { split_args[1].to_owned() } else { negative.unwrap_or("".to_owned()) }.to_owned();

  let (parsed_args, mut argless_prompt) = parse_cli_args(prompt.to_owned());

  /* Argument parsing: style */
  let mut settings = parsed_args.get("style")
    .and_then(|x| serde_json::from_value::<StableDiffusionConfig>(STYLES.get(x.to_owned()).expect("").to_owned()).ok())
    .or_else(|| STYLES.get(style.unwrap_or(String::from("_default"))).and_then(|x| serde_json::from_value::<StableDiffusionConfig>(x.to_owned()).ok()))
    .unwrap_or_default();

    argless_prompt = settings
      .prompt
      .replace("{}", &argless_prompt.trim());
    settings.negative = settings
      .negative
      .replace("{}", &negative);

  /* Argument parsing: size */
  let size_config = globals.config.stable_diffusion.to_owned().sizes.unwrap();
  let size_string = size
    .unwrap_or(parsed_args
      .get("size")
      .unwrap_or(&String::from("square"))
      .parse::<String>()
      .unwrap_or(String::from("square")));
  let size = size_config
    .get(&size_string)
    .unwrap_or(
      size_config.get("_default")
      .expect("default found"));

  settings.width = size.width;
  settings.height = size.height;
  settings.batch_size = size.batch_size;

  let cell_dimensions = (settings.width, settings.height);
  let (rows, columns) = calculate_grid_size(settings.batch_size);
  let grid_dimensions = ((rows * cell_dimensions.0), (columns * cell_dimensions.1));
  let mut grid_img = imagec::ImageBuffer::new(grid_dimensions.0 as u32, grid_dimensions.1 as u32);

  /* Argument parsing: seed */
  if parsed_args.contains_key("seed") || !seed.is_none() {
    settings.seed = seed
    .unwrap_or(parsed_args
      .get("seed")
      .unwrap_or(&String::from("-1"))
      .parse::<i64>()
      .unwrap_or(-1));
  }

  /* Argument parsing: cfg_scale */
  if parsed_args.contains_key("cfg_scale") || !cfg_scale.is_none() {
    settings.cfg_scale = cfg_scale
    .unwrap_or(parsed_args
      .get("cfg_scale")
      .unwrap_or(&String::from("7.5"))
      .parse::<f64>()
      .unwrap_or(7.5));
  }

  /* Argument parsing: restore_faces */
  if parsed_args.contains_key("restore_faces") || !restore_faces.is_none() {
    settings.restore_faces = restore_faces
    .unwrap_or(parsed_args
      .get("restore_faces")
      .unwrap_or(&String::from("true"))
      .parse::<bool>()
      .unwrap_or(true));
  }

  /* Argument parsing: fast */
  if parsed_args.contains_key("fast") || !fast.is_none() {
    let parsed_fast = restore_faces
    .unwrap_or(parsed_args
      .get("restore_faces")
      .unwrap_or(&String::from("true"))
      .parse::<bool>()
      .unwrap_or(true));

      if parsed_fast {
        settings.iterations = settings.iterations / 2;
      }
  }

  settings._api = globals.config.stable_diffusion._api.clone();

  let image_data = stable_diffusion_imagine(argless_prompt, settings)
    .await
    .expect("result");

  let mut i = 0;
  while i < image_data.images.len() as usize {
    // From `base64` to `Vec<u8>` to `DynamicImage`
    let image_bytes = base64::decode(&image_data.images[i as usize]).unwrap();
    let image_dynamic = imagec::load_from_memory(&image_bytes).unwrap();
    // Calculate grid positions
    let (x, y) = ((cell_dimensions.0 * (i % rows)), (cell_dimensions.1 * (i / rows)));
    // Draw `image` on `grid_img`
    imagec::imageops::overlay(&mut grid_img, &image_dynamic, x as i64, y as i64);
    // Add 1 to iterator
    i += 1;
  }

  let mut buf: Vec<u8> = Vec::new();
  let mut writer = Cursor::new(&mut buf);

  grid_img.write_to(&mut writer, imagec::ImageOutputFormat::Png).expect("fail");

  let f = AttachmentType::Bytes { data: Cow::from(&buf), filename: "image.png".to_owned() };

  let param_data= image_data.info.expect("{}");
  let parameters: serde_json::Value = serde_json::from_str(&param_data).expect("data");

  let avatar = match ctx.author().avatar_url() {
    Some(x) =>  x,
    None => ctx.author().default_avatar_url(),
  };

  let finished_time = Utc::now();
  let took = finished_time-start_time;
  let ms_took = took.num_milliseconds();
  // let took = finished_time.signed_duration_since(start_time);
  let format = format!("{}", ms_took / 1000);

  let dimensions = format!("{} x {}", &parameters["width"], &parameters["height"]);

  let author_id: String = format!("{}", ctx.author().id);
  let server_id: String = format!("{}", ctx.guild_id().unwrap_or_default());
  let channel_id: String = format!("{}", ctx.channel_id());
  let img_blob = cdrs_tokio::types::blob::Blob::new(buf.to_vec());

  let job_time = took.num_nanoseconds().unwrap_or(0);

  let image_schema = ImageData {
    id: fuuid,
    flagged: false,
    origin_author: author_id,
    origin_channel: channel_id,
    origin_server: server_id,
    creation: Utc::now(),
    job_time: job_time,
    settings: param_data,
    grid_image: img_blob
  };

  cassandra::query(&database, CQL_INSERT_IMAGE.clone(), image_schema).await;

  let notice_string: String = ctx.data()
    .config.stable_diffusion._notice_string
    .to_owned()
    .unwrap_or("".into());

	ctx.send(|m|
    m.attachment(f)
      .embed(|e| {
        if negative.len() > 0 {
          e.field("Negative", negative, false);
        }
        e
        .color(serenity::utils::Colour::from_rgb(47, 49, 54))
        .title(format!("{}", fuuid))
        .description(format!("{notice_string}\n\nFinished, took a total of `{} seconds`.\nCreated on <t:{}:F> ", format, Utc::now().timestamp()))
        .field("Prompt", ellipsis(prompt, 1024), false)
        .field("Seed", &parameters["seed"], true)
        .field("Dimensions", dimensions, true)
        .field("CFG Scale", &parameters["cfg_scale"], true)
        .footer(|f|
          f
            .text(format!("Requested by {} | ko-fi.com/wessel | {}v{}", ctx.author().tag(), constants::NAME, constants::VERSION))
            .icon_url(avatar))
        .image("attachment://image.png")
      }))
		.await?;

    original_message.delete(ctx).await?;

    Ok(())
}

fn calculate_grid_size(n: usize) -> (usize, usize) {
    let m = (n as f64).sqrt() as usize;
    let r = (n as f64 / m as f64).ceil() as usize;
    let c = (n as f64 / r as f64).ceil() as usize;

    (r, c)
}

async fn stable_diffusion_imagine(prompt: String, settings: StableDiffusionConfig) -> Result<StableDiffusionResponse, Box<dyn std::error::Error>> {
  let request_client = webClient::new();
  let body = request_client.post(format!("{}/sdapi/v1/txt2img", settings._api.expect("")))
    .header(CONTENT_TYPE, "application/json")
    .json(&serde_json::json!({
      "prompt": prompt,
	    "batch_size": settings.batch_size,
      "steps": settings.iterations,
      "width": settings.width,
      "height": settings.height,
      "restore_faces": settings.restore_faces,
      "negative_prompt": settings.negative,
      "seed": settings.seed,
      "cfg_scale": settings.cfg_scale,
    }))
    .send()
    .await?
    .json::<StableDiffusionResponse>()
    .await?;
  Ok(body)
}
