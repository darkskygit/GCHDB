use super::*;

fn check_record(conn: &SqliteConnection, record: &Record) -> ChatRecordResult<bool> {
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
    .get_result(conn)
    .unwrap_or(false))
}

fn update_record(conn: &SqliteConnection, record: &Record) -> ChatRecordResult<usize> {
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
    .execute(conn)?)
}

fn insert_record(conn: &SqliteConnection, record: &Record) -> ChatRecordResult<usize> {
    Ok(insert_into(records::table).values(record).execute(conn)?)
}

pub fn insert_or_update_record(conn: &SqliteConnection, record: &Record) -> ChatRecordResult<bool> {
    Ok(if check_record(&conn, &record)? {
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
