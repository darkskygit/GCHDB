use super::*;

fn check_blob(conn: &mut SqliteConnection, hash: i64) -> ChatRecordResult<bool> {
    use schema::blobs::dsl;
    Ok(select(exists(dsl::blobs.filter(dsl::hash.eq(hash))))
        .get_result(conn)
        .unwrap_or(false))
}

fn insert_blob_inner(conn: &mut SqliteConnection, blob: &Blob) -> ChatRecordResult<usize> {
    Ok(insert_into(blobs::table).values(blob).execute(conn)?)
}

pub fn insert_blob(conn: &mut SqliteConnection, blob: &Blob) -> ChatRecordResult<bool> {
    Ok(if check_blob(conn, blob.hash)? {
        true
    } else {
        insert_blob_inner(conn, blob)? == 1
    })
}

pub fn get_blob(conn: &mut SqliteConnection, blob_hash: i64) -> ChatRecordResult<Vec<u8>> {
    use schema::blobs::dsl::*;
    Ok(blobs
        .filter(hash.eq(blob_hash))
        .select(blob)
        .get_result::<Vec<u8>>(conn)?)
}

pub fn remove_blob(conn: &mut SqliteConnection, hash: i64) -> ChatRecordResult<usize> {
    use schema::blobs::{dsl, table};
    Ok(delete(table).filter(dsl::hash.eq(hash)).execute(conn)?)
}
