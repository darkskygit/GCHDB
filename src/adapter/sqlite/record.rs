use super::*;

fn check_record(conn: &SqliteConnection, record: &Record) -> ChatRecordResult<Option<Vec<u8>>> {
    use schema::records::dsl::*;
    Ok(records
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
        .or_else(|e| (e == DieselError::NotFound).then(|| None).ok_or(e))?)
}

fn update_record(conn: &SqliteConnection, record: &Record) -> ChatRecordResult<usize> {
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

fn insert_record(conn: &SqliteConnection, record: &Record) -> ChatRecordResult<usize> {
    Ok(insert_into(records::table).values(record).execute(conn)?)
}

pub fn insert_or_update_record<F: Fn(Vec<u8>, Vec<u8>) -> Option<Vec<u8>>>(
    conn: &SqliteConnection,
    record: &Record,
    metadata_merger: F,
) -> ChatRecordResult<bool> {
    Ok(if let Some(old_metadata) = check_record(&conn, &record)? {
        let mut record = record.clone();
        if let Some(metadata) = record.metadata {
            record.metadata = metadata_merger(old_metadata, metadata);
        } else {
            record.metadata = Some(old_metadata)
        }
        update_record(conn, &record)
    } else {
        insert_record(conn, &record)
    }? == 1)
}

pub fn remove_record(conn: &SqliteConnection, record: &Record) -> ChatRecordResult<usize> {
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

pub fn remove_record_by_id(conn: &SqliteConnection, id: i32) -> ChatRecordResult<usize> {
    Ok(delete(records::table)
        .filter(records::id.eq(Some(id)))
        .execute(conn)?)
}

pub fn get_record_id(conn: &SqliteConnection, record: &Record) -> ChatRecordResult<i32> {
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
