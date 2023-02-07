mod commands;
mod util;
mod structures;
mod database;
mod events;
mod client;
mod constants;

use log::{info};

#[tokio::main]
async fn main() {
  env_logger::init();

  info!("Connecting to Cassandra/Scylladb instance");
  let database_connection = database::cassandra::connect("127.0.0.1:9042").await;
  database::cassandra::initialize(&database_connection).await;

  /* Connecting to Redis instance */
  info!("Connecting to redis pool");
  let redis_pool = darkredis::ConnectionPool::create(String::from("127.0.0.1:6379"), None, num_cpus::get()).await.expect("");



  let framework = client::init(database_connection, redis_pool).await.expect("framework");

  /* Starting Poise framework */
  framework.run_autosharded().await.unwrap();
}
