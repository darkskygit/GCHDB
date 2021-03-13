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
    fn insert_or_update_record<R>(
        &mut self,
        record: R,
        merger: Option<Box<dyn Fn(&Self, Vec<u8>, Vec<u8>) -> Option<Vec<u8>>>>,
    ) -> ChatRecordResult<bool>
    where
        R: Into<RecordType<'a>>;
    fn remove_record<R: Into<RecordType<'a>>>(&mut self, record: R) -> ChatRecordResult<bool>;
    fn get_record(&self, query: Query) -> ChatRecordResult<Vec<Record>>;
}

#[derive(Clone)]
pub enum RecordType<'a> {
    Id(i32),
    Record(Record),
    RecordRef(&'a Record),
    RecordWithAttachs {
        record: Record,
        attachs: HashMap<String, Vec<u8>>,
    },
    RecordRefWithAttachs {
        record: &'a Record,
        attachs: HashMap<String, Vec<u8>>,
    },
}

impl<'a> RecordType<'a> {
    pub fn get_record(&'a self) -> Option<&'a Record> {
        match self {
            RecordType::Record(record) | RecordType::RecordWithAttachs { record, .. } => {
                Some(&record)
            }
            RecordType::RecordRef(record) | RecordType::RecordRefWithAttachs { record, .. } => {
                Some(record)
            }
            _ => None,
        }
    }
    pub fn display(&self) -> String {
        self.get_record()
            .map(Record::display)
            .unwrap_or("[no content]".into())
    }
}

impl<'a> From<i32> for RecordType<'a> {
    fn from(src: i32) -> Self {
        Self::Id(src)
    }
}

impl From<Record> for RecordType<'_> {
    fn from(src: Record) -> Self {
        Self::Record(src)
    }
}

impl<'a> From<&'a Record> for RecordType<'a> {
    fn from(src: &'a Record) -> Self {
        Self::RecordRef(src)
    }
}

impl From<(Record, HashMap<String, Vec<u8>>)> for RecordType<'_> {
    fn from(src: (Record, HashMap<String, Vec<u8>>)) -> Self {
        Self::RecordWithAttachs {
            record: src.0,
            attachs: src.1,
        }
    }
}

impl<'a> From<(&'a Record, HashMap<String, Vec<u8>>)> for RecordType<'a> {
    fn from(src: (&'a Record, HashMap<String, Vec<u8>>)) -> Self {
        Self::RecordRefWithAttachs {
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
