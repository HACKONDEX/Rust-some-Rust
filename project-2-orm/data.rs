#![forbid(unsafe_code)]

use std::{borrow::Cow, fmt};

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct ObjectId(i64);

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl From<i64> for ObjectId {
    fn from(x: i64) -> Self {
        Self(x)
    }
}

impl ObjectId {
    pub fn into_sql(&self) -> &dyn rusqlite::ToSql {
        &self.0
    }
    pub fn into_i64(&self) -> i64 {
        self.0 as i64
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataType {
    String,
    Bytes,
    Int64,
    Float64,
    Bool,
}

pub trait ObjectColumnType {
    const NAME: DataType;
}

impl ObjectColumnType for i64 {
    const NAME: DataType = DataType::Int64;
}

impl ObjectColumnType for f64 {
    const NAME: DataType = DataType::Float64;
}

impl ObjectColumnType for String {
    const NAME: DataType = DataType::String;
}

impl ObjectColumnType for Vec<u8> {
    const NAME: DataType = DataType::Bytes;
}

impl ObjectColumnType for bool {
    const NAME: DataType = DataType::Bool;
}
////////////////////////////////////////////////////////////////////////////////

pub enum Value<'a> {
    String(Cow<'a, str>),
    Bytes(Cow<'a, [u8]>),
    Int64(i64),
    Float64(f64),
    Bool(bool),
}

impl<'a> Value<'a> {
    pub fn to_sql_from_value(&self) -> &dyn rusqlite::ToSql {
        match self {
            Value::Int64(int) => int,
            Value::Float64(float) => float,
            Value::String(string) => string,
            Value::Bytes(bytes) => bytes,
            Value::Bool(b) => b,
        }
    }
}

pub trait ToSqlRow {
    fn to_sql_row(&self) -> Vec<&dyn rusqlite::ToSql>;
}

// i64 <-> Int64

impl<'a> From<&'a i64> for Value<'static> {
    fn from(f: &'a i64) -> Self {
        Value::Int64(*f)
    }
}

impl<'a> From<Value<'_>> for i64 {
    fn from(x: Value<'_>) -> Self {
        match x {
            Value::Int64(y) => y,
            _ => 0,
        }
    }
}

// f64 <-> Float64

impl<'a> From<&'a f64> for Value<'static> {
    fn from(f: &'a f64) -> Self {
        Value::Float64(*f)
    }
}

impl<'a> From<Value<'_>> for f64 {
    fn from(x: Value<'_>) -> Self {
        match x {
            Value::Float64(y) => y,
            _ => 0.0,
        }
    }
}

// String <-> Value::String

impl<'a> From<&'a String> for Value<'a> {
    fn from(string: &'a String) -> Self {
        Value::String(string.into())
    }
}

impl<'a> From<Value<'_>> for String {
    fn from(x: Value<'_>) -> Self {
        match x {
            Value::String(y) => y.to_string(),
            _ => "".to_string(),
        }
    }
}

// Vec<u8> <-> Bytes

impl<'a> From<&'a Vec<u8>> for Value<'a> {
    fn from(bytes: &'a Vec<u8>) -> Self {
        Value::Bytes(bytes.into())
    }
}

impl<'a> From<Value<'_>> for Vec<u8> {
    fn from(x: Value<'_>) -> Self {
        match x {
            Value::Bytes(y) => y.to_vec(),
            _ => {
                vec![]
            }
        }
    }
}

// bool <-> Bool

impl<'a> From<&'a bool> for Value<'static> {
    fn from(b: &'a bool) -> Self {
        Value::Bool(*b)
    }
}

impl<'a> From<Value<'_>> for bool {
    fn from(x: Value<'_>) -> Self {
        match x {
            Value::Bool(y) => y,
            _ => false,
        }
    }
}
