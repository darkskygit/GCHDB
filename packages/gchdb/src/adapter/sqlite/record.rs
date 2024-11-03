use super::*;

enum RecordExistence {
    Exist(Option<Vec<u8>>),
    NotExist,
}

fn check_record(conn: &mut SqliteConnection, record: &Record) -> ChatRecordResult<RecordExistence> {
    use schema::records::dsl::*;
    match records
        .filter(
            chat_type
                .eq(&record.chat_type)
                .and(owner_id.eq(&record.owner_id))
                .and(group_id.eq(&record.group_id))
                .and(sender_id.eq(&record.sender_id))
                .and(timestamp.eq(record.timestamp)),
        )
        .select(metadata)
        .get_result(conn)
    {
        Ok(data) => Ok(RecordExistence::Exist(data)),
        Err(DieselError::NotFound) => Ok(RecordExistence::NotExist),
        Err(e) => Err(e.into()),
    }
}

fn update_record(conn: &mut SqliteConnection, record: &Record) -> ChatRecordResult<usize> {
    use schema::records::dsl::*;
    Ok(update(
        records.filter(
            id.eq(record.id).or(chat_type
                .eq(&record.chat_type)
                .and(owner_id.eq(&record.owner_id))
                .and(group_id.eq(&record.group_id))
                .and(sender_id.eq(&record.sender_id))
                .and(timestamp.eq(record.timestamp))),
        ),
    )
    .set((
        sender_name.eq(&record.sender_name),
        content.eq(&record.content),
        metadata.eq(&record.metadata),
    ))
    .execute(conn)?)
}

fn insert_record(conn: &mut SqliteConnection, record: &Record) -> ChatRecordResult<usize> {
    Ok(insert_into(records::table).values(record).execute(conn)?)
}

pub fn insert_or_update_record(
    conn: &mut SqliteConnection,
    recorder: &SqliteChatRecorder,
    record: &Record,
    attachs: &HashMap<String, Vec<u8>>,
    metadata_merger: MetadataMerger<SqliteChatRecorder>,
) -> ChatRecordResult<bool> {
    Ok(
        if let RecordExistence::Exist(old_metadata) = check_record(conn, record)? {
            let mut record = record.clone();
            if let Some(metadata) = record.metadata {
                record.metadata = if let Some(old_metadata) = old_metadata {
                    metadata_merger(recorder, attachs, old_metadata, metadata)
                } else {
                    Some(metadata.clone())
                }
            } else {
                record.metadata = old_metadata
            }
            update_record(conn, &record)
        } else {
            insert_record(conn, record)
        }? == 1,
    )
}

pub fn remove_record(conn: &mut SqliteConnection, record: &Record) -> ChatRecordResult<usize> {
    use schema::records::{dsl::*, table};
    Ok(delete(table)
        .filter(
            id.eq(record.id).or(chat_type
                .eq(&record.chat_type)
                .and(owner_id.eq(&record.owner_id))
                .and(group_id.eq(&record.group_id))
                .and(timestamp.eq(record.timestamp))),
        )
        .execute(conn)?)
}

pub fn remove_record_by_id(conn: &mut SqliteConnection, id: i32) -> ChatRecordResult<usize> {
    Ok(delete(records::table)
        .filter(records::id.eq(Some(id)))
        .execute(conn)?)
}

pub fn get_record_id(conn: &mut SqliteConnection, record: &Record) -> ChatRecordResult<i32> {
    use schema::records::dsl::*;
    Ok(records
        .filter(
            id.eq(record.id).or(chat_type
                .eq(&record.chat_type)
                .and(owner_id.eq(&record.owner_id))
                .and(group_id.eq(&record.group_id))
                .and(timestamp.eq(record.timestamp))),
        )
        .select(id)
        .get_result::<Option<i32>>(conn)?
        .unwrap_or(0))
}
