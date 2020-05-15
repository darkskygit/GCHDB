use super::*;
use chrono::Local;
use std::convert::TryInto;

embed_migrations!("migrations");

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

    fn record_search(&self, query: Query) -> ChatRecordResult<Vec<Record>> {
        use schema::records::dsl::*;
        Ok(records
            .filter(
                chat_type
                    .like(query.chat_type.unwrap_or("%%".into()))
                    .and(owner_id.like(query.owner_id.unwrap_or("%%".into())))
                    .and(group_id.like(query.group_id.unwrap_or("%%".into())))
                    .and(sender.like(query.sender.unwrap_or("%%".into())))
                    .and(content.like(query.keyword.unwrap_or("%%".into())))
                    .and(
                        timestamp.le(query
                            .before
                            .unwrap_or(Local::now().naive_utc().timestamp_millis())),
                    )
                    .and(timestamp.ge(query.after.unwrap_or(0))),
            )
            .offset(
                query
                    .offset
                    .unwrap_or_default()
                    .try_into()
                    .unwrap_or(i64::MAX),
            )
            .limit(query.limit.unwrap_or(50).into())
            .load::<Record>(&self.conn.get()?)?)
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
        self.record_search(query)
    }
}

#[test]
fn test_chat_record() -> ChatRecordResult<()> {
    let mut recoder = SqliteChatRecorder::new("record.db")?;
    let record = Record {
        chat_type: "testaasdavxz".into(),
        owner_id: "asdasdasdaaaa".into(),
        group_id: "asdasdasd".into(),
        sender: "哈哈".into(),
        content: "测试".into(),
        timestamp: chrono::Local::now().naive_utc().timestamp_millis(),
        ..Default::default()
    };
    assert_eq!(recoder.insert_or_update_record(&record)?, true);
    println!(
        "{:?}",
        recoder.get_record(Query {
            chat_type: Some("testaasdavxz".into()),
            sender: Some("%哈".into()),
            ..Default::default()
        })?
    );
    assert_eq!(recoder.remove_record(record)?, true);
    Ok(())
}
