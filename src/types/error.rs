use diesel::{r2d2::PoolError, result::Error as DieselError};
use tantivy::{query::QueryParserError, TantivyError};
use thiserror::*;

#[derive(Debug, Error)]
pub enum ChatRecordError {
    #[error(transparent)]
    DatabaseError(#[from] PoolError),
    #[error(transparent)]
    DieselError(#[from] DieselError),
    #[error("{0:?}")]
    TantivyError(TantivyError),
    #[error("{0:?}")]
    TantivyQueryError(QueryParserError),
    #[error(transparent)]
    ContextedError(#[from] anyhow::Error),
}

impl From<TantivyError> for ChatRecordError {
    fn from(src: TantivyError) -> Self {
        Self::TantivyError(src)
    }
}

impl From<QueryParserError> for ChatRecordError {
    fn from(src: QueryParserError) -> Self {
        Self::TantivyQueryError(src)
    }
}
