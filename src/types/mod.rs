mod attach;
mod blob;
mod error;
mod query;
mod record;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;

pub use crate::schema::*;
pub use attach::Attachment;
pub use blob::Blob;
pub use error::ChatRecordError;
pub use query::Query;
pub use record::Record;

pub trait ChatRecoder<'a> {
    fn insert_or_update_record<R: Into<RecordType<'a>>>(
        &mut self,
        record: R,
    ) -> ChatRecordResult<bool>;
    fn remove_record<R: Into<RecordType<'a>>>(&mut self, record: R) -> ChatRecordResult<bool>;
    fn get_record(&self, query: Query) -> ChatRecordResult<Vec<Record>>;
}

pub enum RecordType<'a> {
    Id(i32),
    Record(&'a Record),
    RecordWithAttachs {
        record: &'a Record,
        attachs: HashMap<String, Vec<u8>>,
    },
}

impl<'a> From<i32> for RecordType<'a> {
    fn from(src: i32) -> Self {
        Self::Id(src)
    }
}

impl<'a> From<&'a Record> for RecordType<'a> {
    fn from(src: &'a Record) -> Self {
        Self::Record(src)
    }
}

impl<'a> From<(&'a Record, HashMap<String, Vec<u8>>)> for RecordType<'a> {
    fn from(src: (&'a Record, HashMap<String, Vec<u8>>)) -> Self {
        Self::RecordWithAttachs {
            record: src.0,
            attachs: src.1,
        }
    }
}

pub enum AttachType {
    Id(i32),
    Attach(Attachment),
}

impl From<i32> for AttachType {
    fn from(src: i32) -> Self {
        Self::Id(src)
    }
}

impl From<Attachment> for AttachType {
    fn from(src: Attachment) -> Self {
        Self::Attach(src)
    }
}

pub type ChatRecordResult<T> = Result<T, ChatRecordError>;
