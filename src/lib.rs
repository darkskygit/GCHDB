#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod schema;
mod types;

use anyhow::*;
use diesel::{
    dsl::{delete, exists, insert_into, select, update},
    prelude::*,
    r2d2::{ConnectionManager, Pool},
    sqlite::SqliteConnection,
};
use std::path::Path;
use types::*;

embed_migrations!("migrations");

pub trait ChatRecoder {
    fn insert_or_update_record(&mut self, record: &Record) -> ChatRecordResult<bool>;
    fn remove_record<R: Into<RecordType>>(&mut self, record: R) -> ChatRecordResult<bool>;
    fn get_record(&self, query: Query) -> ChatRecordResult<Vec<Record>>;
}

pub struct SqliteChatRecorder {
    conn: Pool<ConnectionManager<SqliteConnection>>,
}

impl SqliteChatRecorder {
    pub fn new<P: AsRef<Path>>(db_name: P) -> ChatRecordResult<Self> {
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

    pub fn record_exists(&self, record: &Record) -> ChatRecordResult<bool> {
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

    fn record_update(&mut self, record: &Record) -> ChatRecordResult<usize> {
        use schema::records::dsl::*;
        Ok(update(
            records.filter(
                id.eq(record.id).or(chat_type
                    .eq(&record.chat_type)
                    .and(owner_id.eq(&record.owner_id))
                    .and(group_id.eq(&record.group_id))
                    .and(timestamp.eq(record.timestamp))),
            ),
        )
        .set((
            sender.eq(&record.sender),
            content.eq(&record.content),
            metadata.eq(&record.metadata),
        ))
        .execute(&self.conn.get()?)?)
    }

    fn record_insert(&mut self, record: &Record) -> ChatRecordResult<usize> {
        Ok(insert_into(records::table)
            .values(record)
            .execute(&self.conn.get()?)?)
    }

    fn record_remove(&mut self, record: &Record) -> ChatRecordResult<usize> {
        use schema::records::{dsl::*, table};
        Ok(delete(table)
            .filter(
                id.eq(record.id).or(chat_type
                    .eq(&record.chat_type)
                    .and(owner_id.eq(&record.owner_id))
                    .and(group_id.eq(&record.group_id))
                    .and(timestamp.eq(record.timestamp))),
            )
            .execute(&self.conn.get()?)?)
    }

    fn record_remove_by_id(&mut self, id: i32) -> ChatRecordResult<usize> {
        Ok(delete(records::table)
            .filter(records::id.eq(Some(id)))
            .execute(&self.conn.get()?)?)
    }
}

impl ChatRecoder for SqliteChatRecorder {
    fn insert_or_update_record(&mut self, record: &Record) -> ChatRecordResult<bool> {
        Ok(if self.record_exists(&record)? {
            self.record_update(record)
        } else {
            self.record_insert(record)
        }? == 1)
    }

    fn remove_record<R: Into<RecordType>>(&mut self, record: R) -> ChatRecordResult<bool> {
        Ok(match record.into() {
            RecordType::Id(id) => self.record_remove_by_id(id),
            RecordType::Record(record) => self.record_remove(&record),
        }? == 1)
    }

    fn get_record(&self, query: Query) -> ChatRecordResult<Vec<Record>> {
        todo!()
    }
}

#[test]
fn test_chat_record() -> ChatRecordResult<()> {
    let mut recoder = SqliteChatRecorder::new("record.db")?;
    assert_eq!(recoder.insert_or_update_record(&Record::default())?, true);
    assert_eq!(recoder.remove_record(1)?, true);
    Ok(())
}
