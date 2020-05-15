#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod schema;
mod sqlite_adapter;
mod types;

use anyhow::*;
use diesel::{
    dsl::{delete, exists, insert_into, select, update},
    prelude::*,
    r2d2::{ConnectionManager, Pool},
    sqlite::SqliteConnection,
};
use std::path::Path;
use types::*;

pub use sqlite_adapter::SqliteChatRecorder;
pub use types::{ChatRecoder, ChatRecordError, Query};
