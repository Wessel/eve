use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub prefix: String,
    pub cooldowns: CooldownsConfig,
    pub authentication: AuthConfig,
    pub stable_diffusion: StableDiffusionConfig
}

#[derive(Deserialize, Clone, Debug)]
pub struct CooldownsConfig {
  pub _ignore: Vec<u64>,
  pub imagine: u32,
  pub show: u32,
}



// TODO: CREATE SHOW CONFIG
impl Default for StableDiffusionConfig {
    fn default() -> Self {
        StableDiffusionConfig {
            _api: None,
            _notice_string: None,
            prompt: "".into(),
            negative: "".into(),
            seed: 0,
            width: 0,
            height: 0,
            batch_size: 0,
            iterations: 0,
            cfg_scale: 0.0,
            restore_faces: false,
            sizes: None
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct StableDiffusionConfig {
    pub _api: Option<String>,
    pub _notice_string: Option<String>,
    pub prompt: String,
    #[serde(alias = "negative_prompt")]
    pub negative: String,
    pub seed: i64,
    pub width: usize,
    pub height: usize,
    pub batch_size: usize,
    #[serde(alias = "steps")]
    pub iterations: usize,
    pub cfg_scale: f64,
    pub restore_faces: bool,
    pub sizes: Option<HashMap<String, SizeEntry>>
}

#[derive(Deserialize, Clone, Debug)]
pub struct SizeEntry {
    pub width: usize,
    pub height: usize,
    pub batch_size: usize
}


#[derive(Deserialize, Clone, Debug)]
pub struct AuthConfig {
    pub discord: String
}
#[derive(Deserialize, Clone, Debug)]
pub struct StableDiffusionResponse {
    pub parameters: Option<StableDiffusionParameters>,
    pub images: Vec<String>,
    pub info: Option<String>
}

#[derive(Deserialize, Clone, Debug)]
pub struct StableDiffusionParameters {
    pub enable_hr: bool,
    pub denoising_strength: i16,
    pub firstphase_width: i16,
    pub firstphase_height: i16,
    pub prompt: String,
    pub styles: Option<Vec<String>>,
    pub seed: i64,
    pub subseed: i64,
    pub batch_size: i64,
    pub steps: i64
}

use crate::database::cassandra::CassandraConnection;

// #[derive(Debug)]
pub struct GlobalData {
  pub config: Config,
  pub redis_pool: darkredis::ConnectionPool,
  pub start_time: std::time::Instant,
  pub database: CassandraConnection
  // pub database: cassandra_cpp::Session
    // database: sqlx::SqlitePool,
    // godbolt_targets: std::sync::Mutex<godbolt::GodboltTargets>,
}