#![forbid(unsafe_code)]
use crate::Error::{LockConflict, Storage};
use crate::{data::DataType, object::Schema, ObjectId};
use thiserror::Error;

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    NotFound(Box<NotFoundError>),
    #[error(transparent)]
    UnexpectedType(Box<UnexpectedTypeError>),
    #[error(transparent)]
    MissingColumn(Box<MissingColumnError>),
    #[error("database is locked")]
    LockConflict,
    #[error("storage error: {0}")]
    Storage(#[source] Box<dyn std::error::Error>),
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
#[error("object is not found: type '{type_name}', id {object_id}")]
pub struct NotFoundError {
    pub object_id: ObjectId,
    pub type_name: &'static str,
}

impl NotFoundError {
    pub fn new(object_id: ObjectId, type_name: &'static str) -> Self {
        Self {
            object_id,
            type_name,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
#[error(
    "invalid type for {type_name}::{attr_name}: expected equivalent of {expected_type:?}, \
    got {got_type} (table: {table_name}, column: {column_name})"
)]
pub struct UnexpectedTypeError {
    pub type_name: &'static str,
    pub attr_name: &'static str,
    pub table_name: &'static str,
    pub column_name: &'static str,
    pub expected_type: DataType,
    pub got_type: String,
}

impl UnexpectedTypeError {
    pub fn new(
        type_name: &'static str,
        attr_name: &'static str,
        table_name: &'static str,
        column_name: &'static str,
        expected_type: DataType,
        got_type: String,
    ) -> Self {
        Self {
            type_name,
            attr_name,
            table_name,
            column_name,
            expected_type,
            got_type,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
#[error(
    "missing a column for {type_name}::{attr_name} \
    (table: {table_name}, column: {column_name})"
)]
pub struct MissingColumnError {
    pub type_name: &'static str,
    pub attr_name: &'static str,
    pub table_name: &'static str,
    pub column_name: &'static str,
}

impl MissingColumnError {
    pub fn new(
        type_name: &'static str,
        attr_name: &'static str,
        table_name: &'static str,
        column_name: &'static str,
    ) -> Self {
        Self {
            type_name,
            attr_name,
            table_name,
            column_name,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub type Result<T> = std::result::Result<T, Error>;

impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Self {
        match error {
            rusqlite::Error::SqliteFailure(err, _y) => {
                if err.code != rusqlite::ErrorCode::DatabaseBusy {
                    Storage(Box::new(err))
                } else {
                    LockConflict
                }
            }
            other => Storage(Box::new(other)),
        }
    }
}

fn match_patter_with_error(
    pattern: &str,
    text: &str,
    schema: &Schema,
) -> Option<MissingColumnError> {
    if let Some(start) = text.find(pattern) {
        let column_name = text[(pattern.len() + start)..].trim();
        if column_name == "id" {
            return Some(MissingColumnError::new(
                schema.type_name,
                "id",
                schema.table_name,
                "id",
            ));
        }
        for data in schema.info {
            if data.column_name == column_name {
                return Some(MissingColumnError::new(
                    schema.type_name,
                    data.data_name,
                    schema.table_name,
                    data.column_name,
                ));
            }
        }
    }
    None
}

pub fn get_missing_column_error(text: &str, schema: &Schema) -> Option<MissingColumnError> {
    eprintln!("text isssssssssssss = {}", text);
    let error_1 = match_patter_with_error("no such column:", text, schema);
    if error_1.is_some() {
        return error_1;
    }
    let error_2 = match_patter_with_error("has no column named", text, schema);
    if error_2.is_some() {
        return error_2;
    }
    None
}
