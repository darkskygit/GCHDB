# GCHDB aka General chat history database

This crate provides chat record abstraction, used to store chat records extracted from different chat software, and integrated Chinese full-text index.

# Usage

```rust
fn main() -> ChatRecordResult<()> {
    // create an records database
    let mut recoder = SqliteChatRecorder::new("record.db")?;
    // create record, you can extract some record from other im's database
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
    // insert to database
    assert_eq!(recoder.insert_or_update_record(&record)?, true);
    // index the contents of the record
    recoder.refresh_index()?;
    // query record by sql
    println!(
        "{:?}",
        recoder.get_record(Query {
            chat_type: Some("testaasdavxz".into()),
            sender: Some("%日报".into()),
            ..Default::default()
        })?
    );
    // query record by indexer
    println!(
        "{:?}",
        recoder.get_record(Query {
            keyword: Some("技术学校".into()),
            ..Default::default()
        })?
    );
    // remove record in database
    assert_eq!(recoder.remove_record(record)?, true);
    Ok(())
}
```

# Contributing

Welcome pull request :)

# License

AGPL3.0
