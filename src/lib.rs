#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate tantivy;

mod adapter;
mod indexer;
mod schema;
mod types;
mod utils;

use types::*;
use utils::*;

pub use adapter::SqliteChatRecorder;
pub use indexer::ContentIndexer;
pub use types::{ChatRecoder, ChatRecordError, Query, Record, RecordType};
