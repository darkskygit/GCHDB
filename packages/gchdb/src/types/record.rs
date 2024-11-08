use super::*;

#[derive(Queryable, Insertable, Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[diesel(table_name = records)]
pub struct Record {
    pub id: Option<i32>,
    pub chat_type: String,
    pub owner_id: String,
    pub group_id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub content: String,
    pub timestamp: i64,
    pub metadata: Option<Vec<u8>>,
}

impl Record {
    pub fn get_id(&self) -> i32 {
        self.id.unwrap_or_default()
    }
    pub fn display(&self) -> String {
        format!(
            "{} ({}): {}",
            self.sender_name, self.sender_id, self.content
        )
    }
}
