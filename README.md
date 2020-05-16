# GCHDB aka General chat history database (WIP)

This crate provides a record abstraction for storing chat records extracted by different chat software, and provides full-text search feature.

# Usage

```rust
fn main() -> ChatRecordResult<()> {
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
```

# Contributing

Pull request :)
