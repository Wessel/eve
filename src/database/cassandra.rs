use cdrs_tokio::cluster::session::{Session, TcpSessionBuilder, SessionBuilder};
use cdrs_tokio::cluster::{NodeTcpConfigBuilder, TcpConnectionManager};
use cdrs_tokio::frame::message_response::ResponseBody;
use cdrs_tokio::load_balancing::RoundRobinLoadBalancingStrategy;
use cdrs_tokio::query::QueryValues;
use cdrs_tokio::query_values;
use cdrs_tokio::transport::{TransportTcp};

pub type CassandraConnection = Session<TransportTcp, TcpConnectionManager, RoundRobinLoadBalancingStrategy<TransportTcp, TcpConnectionManager>>;

use crate::{database::schema::ImageData};

pub async fn connect(uri: &str) -> CassandraConnection {
    let cluster_config = NodeTcpConfigBuilder::new()
      .with_contact_point(uri.into())
      .build()
      .await
      .unwrap();
     TcpSessionBuilder::new(RoundRobinLoadBalancingStrategy::new(), cluster_config)
        .build()
        .unwrap()
}

pub async fn initialize(session: &CassandraConnection) {
  let queries = vec![
    "CREATE KEYSPACE IF NOT EXISTS eve
        WITH REPLICATION = {
          'class': 'SimpleStrategy',
          'replication_factor': 1
      };",
    "CREATE TABLE IF NOT EXISTS eve.stablediffusion (
          id          UUID,
          flagged     BOOLEAN,
          origin_author  TEXT,
          origin_channel TEXT,
          origin_server  TEXT,
          creation       TIMESTAMP,
          job_time       TIME,
          settings       TEXT,
          grid_image     BLOB,
          PRIMARY KEY    (id)
        )
        WITH gc_grace_seconds = 0;",
  ];

  let mut statements = vec![];

  for query in queries {
    statements.push(
      session
        .prepare(query)
        .await
        .expect("prepare query")
    );
  }

  for statement in statements {
  session
      .exec(&statement)
      .await
      .expect("execute query");
  }
}

pub async fn query(session: &CassandraConnection, query: String, image: ImageData) {
    session
      .query_with_values(query, image.into_query_values())
      .await
      .expect("execute query");
}

use cdrs_tokio::types::prelude::Row;

pub async fn query_search(session: &CassandraConnection, query: String, identifiers: Option<QueryValues>) -> Option<Vec<Row>> {
  let executor;
  if let None = identifiers {
    executor = session.query_with_values(query, query_values!(uuid::Uuid::new_v4())).await;
  } else {
    executor = session.query_with_values(query, identifiers.unwrap()).await;
  }

  let rows = executor
    .expect("execute query")
    .response_body()
    .expect("get body")
    .into_rows()
    .expect("into rows");

  if rows.is_empty() {
    return None;
  }

  Some(rows)
}

pub async fn query_update(session: &CassandraConnection, query: String, identifiers: Option<QueryValues>) -> ResponseBody {
  let executor;
  if let None = identifiers {
    executor = session.query_with_values(query, query_values!(uuid::Uuid::new_v4())).await;
  } else {
    executor = session.query_with_values(query, identifiers.unwrap()).await;
  }

  let rows = executor
    .expect("execute query")
    .response_body()
    .expect("get body");

    rows
}

// pub async fn query_multiple(session: &CassandraConnection, queries: Vec<String>) {
//   let mut statements = vec![];

//   for query in queries {
//     dbg!(&query);
//     statements.push(
//       session
//         .prepare(query)
//         .await
//         .expect("prepare query")
//     );
//   }

//   for statement in statements {
//   let result: Envelope =
//     session
//       .exec(&statement)
//       .await
//       .expect("execute query");

//     dbg!(result);
//   }
// }
