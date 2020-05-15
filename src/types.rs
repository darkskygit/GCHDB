use diesel::{r2d2::PoolError, result::Error as DieselError};
use serde::{Deserialize, Serialize};
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

#[derive(Queryable, Insertable, Serialize, Deserialize, Clone, Default, PartialEq)]
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

pub struct Query {}

pub type ChatRecordResult<T> = Result<T, ChatRecordError>;
