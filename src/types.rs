use diesel::{r2d2::PoolError, result::Error as DieselError};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use thiserror::*;

pub use crate::schema::*;

#[derive(Debug, Error)]
pub enum ChatRecordError {
    #[error(transparent)]
    DatabaseError(#[from] PoolError),
    #[error(transparent)]
    DieselError(#[from] DieselError),
    #[error(transparent)]
    ContextedError(#[from] anyhow::Error),
}

#[derive(Queryable, Insertable, Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[table_name = "records"]
pub struct Record {
    pub(crate) id: Option<i32>,
    pub chat_type: String,
    pub owner_id: String,
    pub group_id: String,
    pub sender: String,
    pub content: String,
    pub timestamp: i64,
    pub metadata: Option<Vec<u8>>,
}

impl Record {
    pub fn get_id(&self) -> i32 {
        self.id.unwrap_or_default()
    }
}

pub trait ChatRecoder {
    fn insert_or_update_record(&mut self, record: &Record) -> ChatRecordResult<bool>;
    fn remove_record<R: Into<RecordType>>(&mut self, record: R) -> ChatRecordResult<bool>;
    fn get_record(&self, query: Query) -> ChatRecordResult<Vec<Record>>;
}

pub enum RecordType {
    Id(i32),
    Record(Record),
}

impl From<i32> for RecordType {
    fn from(src: i32) -> Self {
        Self::Id(src)
    }
}

impl From<Record> for RecordType {
    fn from(src: Record) -> Self {
        Self::Record(src)
    }
}

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

pub type ChatRecordResult<T> = Result<T, ChatRecordError>;
