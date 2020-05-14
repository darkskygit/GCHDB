#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod schema;

use anyhow::*;
use chrono::{DateTime, Utc};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool, PoolError, PooledConnection},
    sqlite::SqliteConnection,
};
use schema::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::*;

embed_migrations!("migrations");

#[derive(Debug, Error)]
pub enum ChatRecordError {
    #[error(transparent)]
    DatabaseError(#[from] PoolError),
    #[error(transparent)]
    ContextedError(#[from] anyhow::Error),
}

#[derive(Queryable, Insertable, Serialize, Deserialize, Clone, Default, PartialEq)]
#[table_name = "records"]
pub struct Record {
    id: Option<i32>,
    pub chat_type: String,
    pub owner_id: String,
    pub group_id: String,
    pub sender: String,
    pub content: String,
    pub timestamp: i64,
    pub metadata: Option<Vec<u8>>,
}

impl Record {
    pub fn get_id(&self) -> i32 {
        self.id.unwrap_or_default()
    }
}

pub trait ChatRecoder {
    fn insert_or_update_record(&mut self, record: &Record) -> Result<(), ChatRecordError>;
    fn remove_record(&mut self, id: i64) -> Result<(), ChatRecordError>;
}

pub struct SqliteChatRecorder {
    conn: Pool<ConnectionManager<SqliteConnection>>,
}

impl SqliteChatRecorder {
    pub fn new<P: AsRef<Path>>(db_name: P) -> Result<Self, ChatRecordError> {
        let manager = ConnectionManager::<SqliteConnection>::new(
            db_name.as_ref().to_str().unwrap_or("sqlite://record.db"),
        );
        let pool = Pool::builder()
            .build(manager)
            .context("Failed to create pool")?;
        let executor = pool.clone().get().unwrap();
        executor
            .execute("PRAGMA journal_mode = WAL;")
            .context("Failed to init WAL mode")?;
        embedded_migrations::run(&executor).context("Failed to init database")?;
        Ok(Self { conn: pool })
    }

    pub fn record_exists(&self, record: &Record) -> Result<bool, ChatRecordError> {
        use diesel::dsl::{exists, select};
        use schema::records::dsl::*;
        Ok(select(exists(
            records.filter(
                chat_type
                    .eq(&record.chat_type)
                    .and(owner_id.eq(&record.owner_id))
                    .and(
                        group_id
                            .eq(&record.group_id)
                            .and(timestamp.eq(record.timestamp)),
                    ),
            ),
        ))
        .get_result(&self.conn.get()?)
        .unwrap_or(false))
    }
}

impl ChatRecoder for SqliteChatRecorder {
    fn insert_or_update_record(&mut self, record: &Record) -> Result<(), ChatRecordError> {
        if self.record_exists(&record)? {
            
        } else {
        }
        Ok(())
    }
    fn remove_record(&mut self, id: i64) -> Result<(), ChatRecordError> {
        Ok(())
    }
}

#[test]
fn test_chat_record() -> Result<(), ChatRecordError> {
    let mut recoder = SqliteChatRecorder::new("test.db")?;
    recoder.insert_or_update_record(&Record::default())?;
    Ok(())
}
