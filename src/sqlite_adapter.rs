use super::*;

embed_migrations!("migrations");

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
        let executor = pool.clone().get().unwrap();
        executor
            .execute("PRAGMA journal_mode = WAL;")
            .context("Failed to init WAL mode")?;
        embedded_migrations::run(&executor).context("Failed to init database")?;
        let mut recoder = Self {
            conn: pool,
            indexer: ContentIndexer::new()?,
        };
        recoder.refresh_index()?;
        Ok(recoder)
    }

    pub fn refresh_index(&mut self) -> ChatRecordResult<()> {
        self.indexer.cleanup_index()?;
        self.indexer.gen_index(self.record_all()?)?;
        Ok(())
    }

    fn record_exists(&self, record: &Record) -> ChatRecordResult<bool> {
        use schema::records::dsl::*;
        Ok(select(exists(
            records.filter(
                chat_type
                    .eq(&record.chat_type)
                    .and(owner_id.eq(&record.owner_id))
                    .and(group_id.eq(&record.group_id))
                    .and(sender.eq(&record.sender))
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
                    .and(sender.eq(&record.sender))
                    .and(timestamp.eq(record.timestamp))),
            ),
        )
        .set((content.eq(&record.content), metadata.eq(&record.metadata)))
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

    fn record_search(&self, offset: i64, limit: i64, keyword: &str) -> ChatRecordResult<Vec<i32>> {
        Ok(self.indexer.search(offset, limit, keyword)?)
    }

    fn record_all(&self) -> ChatRecordResult<Vec<Record>> {
        use schema::records::dsl::*;
        Ok(records.load::<Record>(&self.conn.get()?)?)
    }

    fn record_query(&self, query: Query) -> ChatRecordResult<Vec<Record>> {
        use schema::records::dsl::*;
        let default_query = records.filter(
            timestamp
                .le(query.before.unwrap_or(get_now()))
                .and(timestamp.ge(query.after.unwrap_or(0))),
        );
        Ok(if let Some(keyword) = &query.keyword {
            default_query
                .filter(id.eq_any(self.record_search(
                    query.get_offset(),
                    query.get_limit(),
                    keyword,
                )?))
                .load::<Record>(&self.conn.get()?)?
        } else {
            default_query
                .filter(
                    chat_type
                        .like(query.get_chat_type())
                        .and(owner_id.like(query.get_owner_id()))
                        .and(group_id.like(query.get_group_id()))
                        .and(sender.like(query.get_sender())),
                )
                .offset(query.get_offset())
                .limit(query.get_limit())
                .load::<Record>(&self.conn.get()?)?
        })
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
        self.record_query(query)
    }
}

#[test]
fn test_chat_record() -> ChatRecordResult<()> {
    let mut recoder = SqliteChatRecorder::new("record.db")?;
    let record = Record {
        chat_type: "testaasdavxz".into(),
        owner_id: "asdasdasdaaaa".into(),
        group_id: "asdasdasd".into(),
        sender: "人民日报".into(),
        content:
            "张华考上了北京大学；李萍进了中等技术学校；我在百货公司当售货员：我们都有光明的前途"
                .into(),
        timestamp: chrono::Local::now().naive_utc().timestamp_millis(),
        ..Default::default()
    };
    assert_eq!(recoder.insert_or_update_record(&record)?, true);
    recoder.refresh_index()?;
    println!(
        "{:?}",
        recoder.get_record(Query {
            chat_type: Some("testaasdavxz".into()),
            sender: Some("%日报".into()),
            ..Default::default()
        })?
    );
    println!(
        "{:?}",
        recoder.get_record(Query {
            keyword: Some("技术学校".into()),
            ..Default::default()
        })?
    );
    assert_eq!(recoder.remove_record(record)?, true);
    Ok(())
}
