
use chrono::DateTime;

use uuid::Uuid;
use cdrs_tokio_helpers_derive::TryFromRow;
use cdrs_tokio::{
  query::QueryValues,
};
use cdrs_tokio::query_values;
use cdrs_tokio::types::prelude::{Blob};

#[derive(Debug, TryFromRow)]
pub struct ImageData {
  pub id: Uuid,
  pub flagged: bool,
  pub origin_author: String,
  pub origin_channel: String,
  pub origin_server: String,
  pub creation: DateTime<chrono::Utc>,
  pub job_time: i64,
  pub settings: String,
  pub grid_image: Blob,
}

impl ImageData {
  pub fn into_query_values(self) -> QueryValues {
    query_values!(
      "id" => self.id,
      "flagged" => self.flagged,
      "origin_author" => self.origin_author,
      "origin_channel" => self.origin_channel,
      "origin_server" => self.origin_server,
      "creation" => self.creation,
      "job_time" => self.job_time,
      "settings" => self.settings,
      "grid_image" => self.grid_image
      )
  }

}