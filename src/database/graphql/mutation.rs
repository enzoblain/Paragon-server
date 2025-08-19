use crate::database::{
    database::perform_insert,
    structures::PermissionLevel
};
use common::{
    entities::{
        candle::CandleInput,
        database::DatabaseData,
        session::SessionInput,
        structures::{
            OneDStructuresInput, TwoDStructuresInput
        },
        trend::TrendInput
    }, 
    utils::log::{LogFile, LogLevel}
};

use async_graphql::{Context, Error, Object};
use sqlx::{PgPool, Postgres, QueryBuilder};
use std::sync::Arc;

// Main GraphQL mutation root
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    pub async fn post(&self, ctx: &Context<'_>, data: DatabaseData) -> Result<bool, Error> {
        let permission = ctx.data::<PermissionLevel>()?;
        if *permission != PermissionLevel::Admin {
            return Err(Error::from("Permission denied"));
        }

        let pool = ctx.data::<PgPool>()?;
        let pool = Arc::new(pool.clone());

        let candles = data.candles;
        let candle_insertion = tokio::spawn({
            let pool = Arc::clone(&pool);

            async move {
                insert_candles(pool, &candles).await
            }
        });
        
        let sessions = data.sessions;
        let session_insertion = tokio::spawn({
            let pool = Arc::clone(&pool);

            async move {
                insert_sessions(pool, &sessions).await
            }
        });

        let trends = data.trends;
        let trend_insertion = tokio::spawn({
            let pool = Arc::clone(&pool);

            async move {
                insert_trends(pool, &trends).await
            }
        });

        let one_d_structures = data.one_d_structure;
        let one_d_structure_insertion = tokio::spawn({
            let pool = Arc::clone(&pool);

            async move {
                insert_one_d_structures(pool, &one_d_structures).await
            }
        });

        let two_d_structures = data.two_d_structure;
        let two_d_structure_insertion = tokio::spawn({
            let pool = Arc::clone(&pool);

            async move {
                insert_two_d_structures(pool, &two_d_structures).await
            }
        });

       let result = tokio::try_join!(
            candle_insertion,
            session_insertion,
            trend_insertion,
            one_d_structure_insertion,
            two_d_structure_insertion
        );

        match result {
            Ok((candle_res, session_res, trend_res, one_d_res, two_d_res)) => {
                for (name, res) in [
                    ("candles", &candle_res),
                    ("sessions", &session_res),
                    ("trends", &trend_res),
                    ("one_d_structure", &one_d_res),
                    ("two_d_structure", &two_d_res),
                ] {
                    if let Err(e) = res {
                        LogFile::add_log(LogLevel::Error, &format!("Failed to insert {}: {:?}", name, e)).ok();
                        return Err(Error::from(format!("Failed to insert {}: {:?}", name, e)));
                    }
                }

                Ok(true)
            }
            Err(e) => {
                LogFile::add_log(LogLevel::Error, &format!("Failed to insert data: {}", e)).ok();
                Err(Error::from(format!("Failed to insert data: {}", e)))
            }
        }
    }
}

pub async fn insert_candles(pool: Arc<PgPool>, candles: &[CandleInput]) -> Result<(), Error> {
    if candles.is_empty() {
        return Ok(());
    }

    let res = perform_insert(pool, || {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new("INSERT INTO candles (symbol, timerange, timestamp, open, high, close, low, volume, direction) ");
        query_builder.push_values(candles.iter(), |mut b, candle| {
            b.push_bind(&candle.symbol)
            .push_bind(&candle.timerange)
            .push_bind(&candle.timestamp)
            .push_bind(&candle.open)
            .push_bind(&candle.high)
            .push_bind(&candle.close)
            .push_bind(&candle.low)
            .push_bind(&candle.volume)
            .push_bind(&candle.direction);
        });

        query_builder
    }).await;

    if let Err(e) = res {
        LogFile::add_log(LogLevel::Error, &format!("Failed to insert candles: {}", e)).ok();

        Err(Error::from(format!("Failed to insert candles: {}", e)))
    } else {
        LogFile::add_log(LogLevel::Info, "Candles inserted successfully").ok();

        Ok(())
    }
}

pub async fn insert_sessions(pool: Arc<PgPool>, sessions: &[SessionInput]) -> Result<(), Error> {
    if sessions.is_empty() {
        return Ok(());
    }

    let res = perform_insert(pool, || {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new("INSERT INTO sessions (symbol, label, start_time, end_time, high, low, open, close, volume)");
        query_builder.push_values(sessions.iter(), |mut b, session| {
            b.push_bind(&session.symbol)
             .push_bind(&session.label)
             .push_bind(&session.start_time)
             .push_bind(&session.end_time)
             .push_bind(&session.high)
             .push_bind(&session.low)
             .push_bind(&session.open)
             .push_bind(&session.close)
             .push_bind(&session.volume);
        });

        query_builder
    }).await;

    if let Err(e) = res {
        LogFile::add_log(LogLevel::Error, &format!("Failed to insert sessions: {}", e)).ok();

        Err(Error::from(format!("Failed to insert sessions: {}", e)))
    } else {
        LogFile::add_log(LogLevel::Info, "Sessions inserted successfully").ok();

        Ok(())
    }
}

pub async fn insert_trends(pool: Arc<PgPool>, trends: &[TrendInput]) -> Result<(), Error> {
    if trends.is_empty() {
        return Ok(());
    }

    let res = perform_insert(pool, || {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new("INSERT INTO trends (symbol, timerange, start_time, end_time, direction, high, low)");
        query_builder.push_values(trends.iter(), |mut b, trend| {
            b.push_bind(&trend.symbol)
             .push_bind(&trend.timerange)
             .push_bind(&trend.start_time)
             .push_bind(&trend.end_time)
             .push_bind(&trend.direction)
             .push_bind(&trend.high)
             .push_bind(&trend.low);
        });

        query_builder
    }).await;

    if let Err(e) = res {
        LogFile::add_log(LogLevel::Error, &format!("Failed to insert trends: {}", e)).ok();

        Err(Error::from(format!("Failed to insert trends: {}", e)))
    } else {
        LogFile::add_log(LogLevel::Info, "Trends inserted successfully").ok();

        Ok(())
    }
}

pub async fn insert_one_d_structures(pool: Arc<PgPool>, structures: &[OneDStructuresInput]) -> Result<(), Error> {
    if structures.is_empty() {
        return Ok(());
    }

    let res = perform_insert(pool, || {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new("INSERT INTO one_d_structures (symbol, structure, timerange, timestamp, price, direction)");
        query_builder.push_values(structures.iter(), |mut b, structure| {
            b.push_bind(&structure.symbol)
             .push_bind(&structure.structure)
             .push_bind(&structure.timerange)
             .push_bind(&structure.timestamp)
             .push_bind(&structure.price)
             .push_bind(&structure.direction);
        });

        query_builder
    }).await;

    if let Err(e) = res {
        LogFile::add_log(LogLevel::Error, &format!("Failed to insert one_d_structures: {}", e)).ok();

        Err(Error::from(format!("Failed to insert one_d_structures: {}", e)))
    } else {
        LogFile::add_log(LogLevel::Info, "OneD structures inserted successfully").ok();

        Ok(())
    }
}

pub async fn insert_two_d_structures(pool: Arc<PgPool>, structures: &[TwoDStructuresInput]) -> Result<(), Error> {
    if structures.is_empty() {
        return Ok(());
    }

    let res = perform_insert(pool, || {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new("INSERT INTO two_d_structures (symbol, structure, timerange, timestamp, high, low, direction)");
        query_builder.push_values(structures.iter(), |mut b, structure| {
            b.push_bind(&structure.symbol)
             .push_bind(&structure.structure)
             .push_bind(&structure.timerange)
             .push_bind(&structure.timestamp)
             .push_bind(&structure.high)
             .push_bind(&structure.low)
             .push_bind(&structure.direction);
        });

        query_builder
    }).await;

    if let Err(e) = res {
        LogFile::add_log(LogLevel::Error, &format!("Failed to insert two_d_structures: {}", e)).ok();

        Err(Error::from(format!("Failed to insert two_d_structures: {}", e)))
    } else {
        LogFile::add_log(LogLevel::Info, "TwoD structures inserted successfully").ok();

        Ok(())
    }
}