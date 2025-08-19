use crate::database::{
    graphql::{
        graphql::{graphiql, graphql_handler},
        mutation::MutationRoot,
        query::QueryRoot
    },
    rest::rest::hello_rest
};

use async_graphql::{ 
    EmptySubscription, 
    Schema
};
use axum::{
    routing::get, Router
};
use common::utils::log::{
    LogFile, 
    LogLevel
};
use sqlx::{
    PgPool,
    postgres::{
        PgPoolOptions, 
        Postgres, 
    },  
    QueryBuilder
};
use std::sync::Arc;
use tokio::{
    net::TcpListener,
    time::{Duration, sleep},
};

pub async fn launch_database(adress: String, database_url: String) -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url).await?;

    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(pool)
        .finish();

    let app = Router::new()
        .route("/data", get(graphiql).post(graphql_handler)) // GraphQL interface
        .route("/api", get(hello_rest).post(hello_rest)) // REST endpoint (GET/POST)
        .with_state(Arc::new(schema));

    let listener = TcpListener::bind(&adress).await;
    if let Err(e) = listener {
        LogFile::add_log(LogLevel::Error, &format!("Failed to bind to {}: {}", adress, e)).ok();

        return Err(Box::new(e));
    }

    let listener = listener.unwrap();

    if let Err(e) = axum::serve(listener, app).into_future().await {
        LogFile::add_log(LogLevel::Error, &format!("Failed to start server: {}", e)).ok();

        return Err(Box::new(e));
    }

    LogFile::add_log(LogLevel::Info, &format!("Database server running on {}", adress)).ok();

    Ok(())
}

pub async fn perform_insert<'a, F>(pool: Arc<PgPool>, build_query_builder: F) -> Result<u64, sqlx::Error> 
where F: Fn() -> QueryBuilder<'a, Postgres> {
    let mut last_err = None;

    for attempt in 0..5 {
        let mut query_builder = build_query_builder();
        let query = query_builder.build();

        match query.execute(pool.as_ref()).await {
            Ok(result) => return Ok(result.rows_affected()),
            Err(e) => {
                query_builder.reset();
                last_err = Some(e);

                sleep(Duration::from_millis(100 * attempt)).await;
            }
        }
    }

    Err(last_err.unwrap_or_else(|| sqlx::Error::Protocol("Unknown error".into())))
}