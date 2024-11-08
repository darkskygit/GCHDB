mod attach;
mod blob;
mod record;

use super::*;
use attach::{insert_or_update_attach, remove_attachs};
use blob::{get_blob, insert_blob, remove_blob};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use record::{get_record_id, insert_or_update_record, remove_record, remove_record_by_id};
use std::collections::HashMap;

use anyhow::Context;
use diesel::{
    connection::SimpleConnection,
    dsl::{delete, exists, insert_into, select, update},
    prelude::*,
    r2d2::{ConnectionManager, Pool},
    result::Error as DieselError,
    sqlite::SqliteConnection,
};
use std::path::Path;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub struct SqliteChatRecorder {
    conn: Pool<ConnectionManager<SqliteConnection>>,
    indexer: ContentIndexer,
}

impl SqliteChatRecorder {
    pub fn new<P: AsRef<Path>>(db_name: P) -> ChatRecordResult<Self> {
        let manager = ConnectionManager::<SqliteConnection>::new(
            db_name.as_ref().to_str().unwrap_or("sqlite://record.db"),
        );
        let pool = Pool::builder()
            .build(manager)
            .context("Failed to create pool")?;
        let mut executor = pool.get().unwrap();
        executor
            .batch_execute("PRAGMA journal_mode = WAL;")
            .context("Failed to init WAL mode")?;
        executor
            .run_pending_migrations(MIGRATIONS)
            .expect("Failed to init database");
        let mut recorder = Self {
            conn: pool,
            indexer: ContentIndexer::new()?,
        };
        recorder.refresh_index()?;
        Ok(recorder)
    }

    pub fn refresh_index(&mut self) -> ChatRecordResult<()> {
        self.indexer.cleanup_index()?;
        self.indexer.gen_index(self.record_all()?)?;
        Ok(())
    }

    fn record_all(&self) -> ChatRecordResult<Vec<Record>> {
        use schema::records::dsl::*;
        Ok(records.load::<Record>(&mut self.conn.get()?)?)
    }

    fn record_query(&self, query: Query) -> ChatRecordResult<Vec<Record>> {
        use schema::records::dsl::*;
        let default_query = records.filter(
            timestamp
                .le(query.before.unwrap_or_else(get_now))
                .and(timestamp.ge(query.after.unwrap_or(0))),
        );
        Ok(if let Some(keyword) = &query.keyword {
            default_query
                .filter(id.eq_any(self.indexer.search(
                    query.get_offset(),
                    query.get_limit(),
                    keyword,
                )?))
                .load::<Record>(&mut self.conn.get()?)?
        } else {
            default_query
                .filter(
                    chat_type
                        .like(query.get_chat_type())
                        .and(owner_id.like(query.get_owner_id()))
                        .and(group_id.like(query.get_group_id()))
                        .and(sender_id.like(query.get_sender_id()))
                        .and(sender_name.like(query.get_sender_name())),
                )
                .offset(query.get_offset())
                .limit(query.get_limit())
                .load::<Record>(&mut self.conn.get()?)?
        })
    }

    fn record_auto_insert(
        &mut self,
        record: &Record,
        attachs: HashMap<String, Vec<u8>>,
        merger: MetadataMerger<Self>,
    ) -> ChatRecordResult<bool> {
        let mut conn = self.conn.get()?;
        Ok(
            insert_or_update_record(&mut conn, self, record, &attachs, merger)? && {
                let mut len = 0;
                let record_id = get_record_id(&mut conn, record)?;
                if record_id > 0 {
                    for (name, blob) in attachs.iter() {
                        insert_or_update_attach(&mut conn, blob.clone(), name.clone(), record_id)?;
                        len += 1;
                    }
                    len
                } else {
                    0
                }
            } == attachs
                .len(),
        )
    }

    pub fn get_blob(&self, hash: i64) -> ChatRecordResult<Vec<u8>> {
        let mut conn = self.conn.get()?;
        get_blob(&mut conn, hash)
    }
}

fn default_metadata_merger(
    _recorder: &SqliteChatRecorder,
    _attachs: &Attachments,
    _old: Vec<u8>,
    new: Vec<u8>,
) -> Option<Vec<u8>> {
    Some(new)
}

impl<'a> ChatRecorder<'a> for SqliteChatRecorder {
    fn insert_or_update_record<R>(
        &mut self,
        record: R,
        merger: Option<MetadataMerger<Self>>,
    ) -> ChatRecordResult<bool>
    where
        R: Into<RecordType<'a>>,
    {
        let merger = merger.unwrap_or(default_metadata_merger);
        Ok(match record.into() {
            RecordType::Id(_) => false,
            RecordType::Record(record) => {
                self.record_auto_insert(&record, Default::default(), merger)?
            }
            RecordType::RecordRef(record) => {
                self.record_auto_insert(record, Default::default(), merger)?
            }
            RecordType::RecordWithAttaches {
                record,
                attaches: attachs,
            } => self.record_auto_insert(&record, attachs, merger)?,
            RecordType::RecordRefWithAttaches {
                record,
                attaches: attachs,
            } => self.record_auto_insert(record, attachs, merger)?,
        })
    }

    fn remove_record<R: Into<RecordType<'a>>>(&mut self, record: R) -> ChatRecordResult<bool> {
        let mut conn = self.conn.get()?;
        Ok(match record.into() {
            RecordType::Id(id) => remove_record_by_id(&mut conn, id)? == 1,
            RecordType::Record(record) | RecordType::RecordWithAttaches { record, .. } => {
                remove_record(&mut conn, &record)? == 1
                    && remove_attachs(&mut conn, record.get_id())?
            }
            RecordType::RecordRef(record) | RecordType::RecordRefWithAttaches { record, .. } => {
                remove_record(&mut conn, record)? == 1
                    && remove_attachs(&mut conn, record.get_id())?
            }
        })
    }

    fn get_record(&self, query: Query) -> ChatRecordResult<Vec<Record>> {
        self.record_query(query)
    }
}

#[test]
fn test_chat_record() {
    let mut recorder = SqliteChatRecorder::new("record.db").unwrap();
    let record = Record {
        chat_type: "testaasdavxz".into(),
        owner_id: "asdasdasdaaaa".into(),
        group_id: "asdasdasd".into(),
        sender_id: "people_daily".into(),
        sender_name: "人民日报".into(),
        content:
            "张华考上了北京大学；李萍进了中等技术学校；我在百货公司当售货员：我们都有光明的前途"
                .into(),
        timestamp: chrono::Local::now()
            .naive_utc()
            .and_utc()
            .timestamp_millis(),
        ..Default::default()
    };
    assert_eq!(
        recorder.insert_or_update_record(&record, None).unwrap(),
        true
    );
    let record1 = Record {
        chat_type: "testaasdavxz".into(),
        owner_id: "asdasdasdaaaa".into(),
        group_id: "asdasdasd".into(),
        sender_id: "news".into(),
        sender_name: "新闻".into(),
        content: "Intel线路图显示他们想恢复两年升级一次工艺，2029年有1.4nm".into(),
        timestamp: chrono::Local::now()
            .naive_utc()
            .and_utc()
            .timestamp_millis(),
        ..Default::default()
    };
    assert_eq!(
        recorder
            .insert_or_update_record(
                (
                    &record1,
                    [("test".into(), vec![0, 1, 2, 3])]
                        .iter()
                        .cloned()
                        .collect()
                ),
                None
            )
            .unwrap(),
        true
    );
    recorder.refresh_index().unwrap();
    println!(
        "{:?}",
        recorder
            .get_record(Query {
                chat_type: Some("testaasdavxz".into()),
                sender_name: Some("%日报".into()),
                ..Default::default()
            })
            .unwrap()
    );
    println!(
        "{:?}",
        recorder
            .get_record(Query {
                keyword: Some("技术学校".into()),
                ..Default::default()
            })
            .unwrap()
    );
    assert_eq!(recorder.remove_record(&record).unwrap(), true);
}
