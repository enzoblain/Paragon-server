use crate::{database::structures::PermissionLevel, Candle, OneDStructures, Session, Trend, TwoDStructures};

use async_graphql::{Context, Error, Interface, Object, SimpleObject};
use common::utils::log::{
    LogFile, LogLevel,
};
use sqlx::{PgPool, query_as};
use std::sync::Arc;
use tokio::try_join;

// GraphQL interface for entities
// So we can facilitate polymorphic queries
// and return different types of objects
#[derive(Interface)]
#[graphql(field(name = "symbol", ty = "&str"))]
#[graphql(field(name = "timerange", ty = "&str"))]
#[graphql(field(name = "start_time", ty = "i64"))]
#[graphql(field(name = "end_time", ty = "i64"))]
pub enum CommonFields {
    Candle(Candle),
    Session(Session),
    Trend(Trend),
    TwoDStructures(TwoDStructures),
    OneDStructures(OneDStructures),
}

// This struct is used to return all common fields in a single query
// It allows us to return different types of entities
#[derive(SimpleObject)]
pub struct AllCommonFieldsResult {
    pub candles: Vec<Candle>,
    pub one_d_structures: Vec<OneDStructures>,
    pub trends: Vec<Trend>,
    pub two_d_structures: Vec<TwoDStructures>,
    pub sessions: Vec<Session>,
}

// Main GraphQL query root
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    pub async fn get(&self, ctx: &Context<'_>, symbol: String, timerange: String, min_timestamp: Option<i64>, max_timestamp: Option<i64>, limit: Option<i64>) -> Result<AllCommonFieldsResult, Error> {
        // Check user permissions
        // But in this case, we allow everyone to access this query
        let permission = ctx.data::<PermissionLevel>()?;
        if *permission != PermissionLevel::Admin && *permission != PermissionLevel::User {
            return Err(Error::from("Permission denied"));
        }

        let pool = ctx.data::<PgPool>()?;
        let pool = Arc::new(pool.clone());

        let sybmol = Arc::new(symbol);
        let timerange = Arc::new(timerange);
        let min_timestamp = Arc::new(min_timestamp);
        let max_timestamp = Arc::new(max_timestamp);
        let limit = Arc::new(limit.unwrap_or(100));

        let candle_selection = tokio::spawn({
            let pool = Arc::clone(&pool);

            let symbol = Arc::clone(&sybmol);
            let timerange = Arc::clone(&timerange);
            let min_timestamp = Arc::clone(&min_timestamp);
            let max_timestamp = Arc::clone(&max_timestamp);
            let limit = Arc::clone(&limit);

            async move {
                let res = select_candles(pool, symbol, timerange.into(), min_timestamp, max_timestamp, limit).await;

                if let Err(ref e) = res {
                    LogFile::add_log(LogLevel::Error, &format!("Failed to retrieve candles: {}", e)).ok();
                }

                res
            }
        });

        let session_selection = tokio::spawn({
            let pool = Arc::clone(&pool);

            let symbol = Arc::clone(&sybmol);
            let min_timestamp = Arc::clone(&min_timestamp);
            let max_timestamp = Arc::clone(&max_timestamp);
            let limit = Arc::clone(&limit);

            async move {
                let res = select_sessions(pool, symbol, min_timestamp, max_timestamp, limit).await;

                if let Err(ref e) = res {
                    LogFile::add_log(LogLevel::Error, &format!("Failed to retrieve sessions: {}", e)).ok();
                }

                res
            }
        });

        let trend_selection = tokio::spawn({
            let pool = Arc::clone(&pool);

            let symbol = Arc::clone(&sybmol);
            let timerange = Arc::clone(&timerange);
            let min_timestamp = Arc::clone(&min_timestamp);
            let max_timestamp = Arc::clone(&max_timestamp);
            let limit = Arc::clone(&limit);

            async move {
                let res = select_trends(pool, symbol, timerange, min_timestamp, max_timestamp, limit).await;

                if let Err(ref e) = res {
                    LogFile::add_log(LogLevel::Error, &format!("Failed to retrieve trends: {}", e)).ok();
                }

                res
            }
        });

        let twodstructure_selection = tokio::spawn({
            let pool = Arc::clone(&pool);

            let symbol = Arc::clone(&sybmol);
            let timerange = Arc::clone(&timerange);
            let min_timestamp = Arc::clone(&min_timestamp);
            let max_timestamp = Arc::clone(&max_timestamp);
            let limit = Arc::clone(&limit);

            async move {
                let res = select_two_d_structures(pool, symbol, timerange, min_timestamp, max_timestamp, limit).await;

                if let Err(ref e) = res {
                    LogFile::add_log(LogLevel::Error, &format!("Failed to retrieve 2D structures: {}", e)).ok();
                }

                res
            }
        });

        let onedstructure_selection = tokio::spawn({
            let pool = Arc::clone(&pool);

            let symbol = Arc::clone(&sybmol);
            let timerange = Arc::clone(&timerange);
            let min_timestamp = Arc::clone(&min_timestamp);
            let max_timestamp = Arc::clone(&max_timestamp);
            let limit = Arc::clone(&limit);

            async move {
                let res = select_one_d_structures(pool, symbol, timerange, min_timestamp, max_timestamp, limit).await;

                if let Err(ref e) = res {
                    LogFile::add_log(LogLevel::Error, &format!("Failed to retrieve 1D structures: {}", e)).ok();
                }

                res
            }
        });

        // Spawn all tasks simultaneously and wait for all to complete
        // Even if the pool as a size of 5, we can still win time
        let (candles, sessions, trends, two_d_structures, one_d_structures) = try_join!(
            candle_selection,
            session_selection,
            trend_selection,
            twodstructure_selection,
            onedstructure_selection
        )?;

        let mut fail = false;
        if candles.is_err() || sessions.is_err() || trends.is_err() || two_d_structures.is_err() || one_d_structures.is_err() {
            fail = true;
        }

        let all = AllCommonFieldsResult {
            candles: candles?,
            one_d_structures: one_d_structures?,
            trends: trends?,
            two_d_structures: two_d_structures?,
            sessions: sessions?,
        };

        if !fail {
            LogFile::add_log(LogLevel::Info, "All data retrieved successfully").map_err(|e| Error::from(format!("Failed to log: {}", e)))?;
        }

        Ok(all)
    }
}

pub async fn select_candles(pool: Arc<PgPool>, symbol: Arc<String>, timerange: Arc<String>, min_timestamp: Arc<Option<i64>>, max_timestamp: Arc<Option<i64>>, limit: Arc<i64>) -> Result<Vec<Candle>, sqlx::Error> {
    query_as::<_, Candle>(r#"
        SELECT symbol, timerange, timestamp, open, high, low, close, volume, direction
        FROM candles
        WHERE symbol = $1
            AND ($2 IS NULL OR timerange = $2)
            AND ($4 IS NULL OR (EXTRACT(EPOCH FROM timestamp) > $4))
            AND ($5 IS NULL OR EXTRACT(EPOCH FROM timestamp) < $5)
        ORDER BY timestamp DESC
        LIMIT $3
    "#)
    .bind(symbol.as_ref())
    .bind(timerange.as_ref())
    .bind(limit.as_ref())
    .bind(min_timestamp.as_ref())
    .bind(max_timestamp.as_ref())
    .fetch_all(pool.as_ref())
    .await
}

pub async fn select_sessions(pool: Arc<PgPool>, symbol: Arc<String>, min_timestamp: Arc<Option<i64>>, max_timestamp: Arc<Option<i64>>, limit: Arc<i64>) -> Result<Vec<Session>, sqlx::Error> {
    query_as::<_, Session>(r#"
        SELECT symbol, label, start_time, end_time, high, low, open, close, volume
        FROM sessions
        WHERE symbol = $1
            AND ($2 IS NULL OR (EXTRACT(EPOCH FROM start_time) > $2))
            AND ($3 IS NULL OR EXTRACT(EPOCH FROM end_time) < $3)
        ORDER BY start_time DESC
        LIMIT $4
    "#)
    .bind(symbol.as_ref())
    .bind(min_timestamp.as_ref())
    .bind(max_timestamp.as_ref())
    .bind(limit.as_ref())
    .fetch_all(pool.as_ref())
    .await
}

pub async fn select_trends(pool: Arc<PgPool>, symbol: Arc<String>, timerange: Arc<String>, min_timestamp: Arc<Option<i64>>, max_timestamp: Arc<Option<i64>>, limit: Arc<i64>) -> Result<Vec<Trend>, sqlx::Error> {
    query_as::<_, Trend>(r#"
        SELECT symbol, timerange, start_time, end_time, direction, high, low
        FROM trends
        WHERE symbol = $1
            AND timerange = $2
            AND ($3 IS NULL OR (EXTRACT(EPOCH FROM start_time) > $3))
            AND ($4 IS NULL OR EXTRACT(EPOCH FROM end_time) < $4)
        ORDER BY start_time DESC
        LIMIT $5
    "#)
    .bind(symbol.as_ref())
    .bind(timerange.as_ref())
    .bind(min_timestamp.as_ref())
    .bind(max_timestamp.as_ref())
    .bind(limit.as_ref())
    .fetch_all(pool.as_ref())
    .await
}

pub async fn select_two_d_structures(pool: Arc<PgPool>, symbol: Arc<String>, timerange: Arc<String>, min_timestamp: Arc<Option<i64>>, max_timestamp: Arc<Option<i64>>, limit: Arc<i64>) -> Result<Vec<TwoDStructures>, sqlx::Error> {
    query_as::<_, TwoDStructures>(r#"
        SELECT symbol, structure, timerange, timestamp, high, low, direction
        FROM two_d_structures
        WHERE symbol = $1
            AND timerange = $2
            AND ($3 IS NULL OR (EXTRACT(EPOCH FROM timestamp) > $3))
            AND ($4 IS NULL OR EXTRACT(EPOCH FROM timestamp) < $4)
        ORDER BY timestamp DESC
        LIMIT $5
    "#)
    .bind(symbol.as_ref())
    .bind(timerange.as_ref())
    .bind(min_timestamp.as_ref())
    .bind(max_timestamp.as_ref())
    .bind(limit.as_ref())
    .fetch_all(pool.as_ref())
    .await
}

pub async fn select_one_d_structures(pool: Arc<PgPool>, symbol: Arc<String>, timerange: Arc<String>, min_timestamp: Arc<Option<i64>>, max_timestamp: Arc<Option<i64>>, limit: Arc<i64>) -> Result<Vec<OneDStructures>, sqlx::Error> {
    query_as::<_, OneDStructures>(r#"
        SELECT symbol, structure, timerange, timestamp, price, direction
        FROM one_d_structures
        WHERE symbol = $1
            AND timerange = $2
            AND ($3 IS NULL OR (EXTRACT(EPOCH FROM timestamp) > $3))
            AND ($4 IS NULL OR EXTRACT(EPOCH FROM timestamp) < $4)
        ORDER BY timestamp DESC
        LIMIT $5
    "#)
    .bind(symbol.as_ref())
    .bind(timerange.as_ref())
    .bind(min_timestamp.as_ref())
    .bind(max_timestamp.as_ref())
    .bind(limit.as_ref())
    .fetch_all(pool.as_ref())
    .await
}