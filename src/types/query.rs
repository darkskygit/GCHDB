use super::*;

#[derive(Clone, Debug, Default)]
pub struct Query {
    pub chat_type: Option<String>,
    pub owner_id: Option<String>,
    pub group_id: Option<String>,
    pub sender: Option<String>,
    pub keyword: Option<String>,
    pub before: Option<i64>,
    pub after: Option<i64>,
    pub offset: Option<u64>,
    pub limit: Option<u32>,
}

impl Query {
    pub fn get_chat_type(&self) -> &str {
        self.chat_type.as_deref().unwrap_or("%%")
    }

    pub fn get_owner_id(&self) -> &str {
        self.owner_id.as_deref().unwrap_or("%%")
    }

    pub fn get_group_id(&self) -> &str {
        self.group_id.as_deref().unwrap_or("%%")
    }

    pub fn get_sender(&self) -> &str {
        self.sender.as_deref().unwrap_or("%%")
    }

    pub fn get_offset(&self) -> i64 {
        self.offset
            .unwrap_or_default()
            .try_into()
            .unwrap_or(i64::MAX)
    }

    pub fn get_limit(&self) -> i64 {
        self.limit.unwrap_or(50).into()
    }
}
