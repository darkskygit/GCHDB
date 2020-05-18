use super::*;

fn check_attach(conn: &SqliteConnection, attach: &Attachment) -> ChatRecordResult<bool> {
    use schema::attachments::dsl::*;
    Ok(select(exists(attachments.filter(
        record_id.eq(&attach.record_id).and(name.eq(&attach.name)),
    )))
    .get_result(conn)
    .unwrap_or(false))
}

fn update_attach(conn: &SqliteConnection, attach: &Attachment) -> ChatRecordResult<usize> {
    use schema::attachments::dsl::*;
    Ok(
        update(attachments.filter(record_id.eq(&attach.record_id).and(name.eq(&attach.name))))
            .set(hash.eq(&attach.hash))
            .execute(conn)?,
    )
}

fn insert_attach(conn: &SqliteConnection, attach: &Attachment) -> ChatRecordResult<usize> {
    Ok(insert_into(attachments::table)
        .values(attach)
        .execute(conn)?)
}

fn remove_attach(conn: &SqliteConnection, attach: &Attachment) -> ChatRecordResult<usize> {
    use schema::attachments::{dsl::*, table};
    Ok(delete(table)
        .filter(
            id.eq(attach.id)
                .or(record_id.eq(&attach.record_id).and(name.eq(&attach.name))),
        )
        .execute(conn)?)
}

fn get_attachs(conn: &SqliteConnection, record_id: i32) -> ChatRecordResult<Vec<Attachment>> {
    use schema::attachments::dsl;
    Ok(dsl::attachments
        .filter(dsl::record_id.eq(record_id))
        .load(conn)?)
}

pub fn remove_attachs(conn: &SqliteConnection, record_id: i32) -> ChatRecordResult<bool> {
    let attachs = get_attachs(&conn, record_id)?;
    Ok(attachs
        .iter()
        .filter(|attach| remove_attach(&conn, attach).unwrap_or(0) == 1)
        .count()
        == attachs.len())
}

pub fn insert_or_update_attach_inner(
    conn: &SqliteConnection,
    attach: &Attachment,
) -> ChatRecordResult<bool> {
    Ok(if check_attach(&conn, &attach)? {
        update_attach(conn, &attach)
    } else {
        insert_attach(conn, &attach)
    }? == 1)
}

pub fn insert_or_update_attach(
    conn: &SqliteConnection,
    data: Vec<u8>,
    name: String,
    record_id: i32,
) -> ChatRecordResult<bool> {
    let blob = Blob::new(data);
    if insert_blob(&conn, &blob)? {
        Ok(insert_or_update_attach_inner(
            &conn,
            &Attachment::new(blob.hash, name, record_id),
        )?)
    } else {
        Ok(false)
    }
}
