#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod schema;

use anyhow::*;
use diesel::{
    dsl::{exists, insert_into, select, update},
    prelude::*,
    r2d2::{ConnectionManager, Pool, PoolError},
    result::Error as DieselError,
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
    DieselError(#[from] DieselError),
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
    fn insert_or_update_record(&mut self, record: &Record) -> Result<bool, ChatRecordError>;
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
        use schema::records::dsl::*;
        Ok(select(exists(
            records.filter(
                chat_type
                    .eq(&record.chat_type)
                    .and(owner_id.eq(&record.owner_id))
                    .and(group_id.eq(&record.group_id))
                    .and(timestamp.eq(record.timestamp)),
            ),
        ))
        .get_result(&self.conn.get()?)
        .unwrap_or(false))
    }

    fn record_update(&mut self, record: &Record) -> Result<usize, ChatRecordError> {
        use schema::records::dsl::*;
        Ok(update(
            records.filter(
                chat_type
                    .eq(&record.chat_type)
                    .and(owner_id.eq(&record.owner_id))
                    .and(group_id.eq(&record.group_id))
                    .and(timestamp.eq(record.timestamp)),
            ),
        )
        .set((
            sender.eq(&record.sender),
            content.eq(&record.content),
            metadata.eq(&record.metadata),
        ))
        .execute(&self.conn.get()?)?)
    }

    fn record_insert(&mut self, record: &Record) -> Result<usize, ChatRecordError> {
        Ok(insert_into(records::table)
            .values(record)
            .execute(&self.conn.get()?)?)
    }
}

impl ChatRecoder for SqliteChatRecorder {
    fn insert_or_update_record(&mut self, record: &Record) -> Result<bool, ChatRecordError> {
        Ok(if self.record_exists(&record)? {
            self.record_update(record)
        } else {
            self.record_insert(record)
        }? == 1)
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
