use super::*;

#[derive(Queryable, Insertable, Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[diesel(table_name = attachments)]
pub struct Attachment {
    pub id: Option<i32>,
    pub record_id: i32,
    pub name: String,
    pub hash: i64,
}

impl<'a> Attachment {
    pub fn new<R: Into<RecordType<'a>>>(hash: i64, name: String, record: R) -> Self {
        Self {
            id: None,
            record_id: match record.into() {
                RecordType::Id(id) => id,
                RecordType::Record(record) | RecordType::RecordWithAttaches { record, .. } => {
                    record.get_id()
                }
                RecordType::RecordRef(record) | RecordType::RecordRefWithAttaches { record, .. } => {
                    record.get_id()
                }
            },
            name,
            hash,
        }
    }

    pub fn get_id(&self) -> i32 {
        self.id.unwrap_or_default()
    }
}
