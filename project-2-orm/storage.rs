#![forbid(unsafe_code)]
use crate::error::NotFoundError;
use crate::Error::{MissingColumn, NotFound, UnexpectedType};
use crate::{
    data::{DataType, Value},
    error::*,
    object::Schema,
    ObjectId,
};
use rusqlite::{params, params_from_iter, ToSql};
use std::borrow::Cow;
use crate::data::ToSqlRow;

////////////////////////////////////////////////////////////////////////////////

pub type Row<'a> = Vec<Value<'a>>;
pub type RowSlice<'a> = [Value<'a>];

impl<'a> ToSqlRow for RowSlice<'a> {
    fn to_sql_row(&self) -> Vec<&dyn ToSql> {
        let mut row = Vec::<&dyn ToSql>::new();
        for value in self {
            row.push(match value {
                Value::Int64(int) => int,
                Value::Float64(float) => float,
                Value::String(string) => string,
                Value::Bytes(bytes) => bytes,
                Value::Bool(b) => b,
            });
        }
        row
    }
}

////////////////////////////////////////////////////////////////////////////////

pub(crate) trait StorageTransaction {
    fn table_exists(&self, table: &str) -> Result<bool>;
    fn create_table(&self, schema: &Schema) -> Result<()>;

    fn insert_row(&self, schema: &Schema, row: &RowSlice) -> Result<ObjectId>;
    fn update_row(&self, id: ObjectId, schema: &Schema, row: &RowSlice) -> Result<()>;
    fn select_row(&self, id: ObjectId, schema: &Schema) -> Result<Row<'static>>;
    fn delete_row(&self, id: ObjectId, schema: &Schema) -> Result<()>;

    fn commit(&self) -> Result<()>;
    fn rollback(&self) -> Result<()>;
}

impl<'a> StorageTransaction for rusqlite::Transaction<'a> {
    fn table_exists(&self, table: &str) -> Result<bool> {
        let query = self.prepare("SELECT 1 FROM sqlite_master WHERE name = ?");
        Ok(query?.query_row(params![table], |_| Ok(())).is_ok())
    }

    fn create_table(&self, schema: &Schema) -> Result<()> {
        self.execute(schema.make_create_query_str().as_str(), params![])?;
        Ok(())
    }

    fn insert_row(&self, schema: &Schema, row: &RowSlice) -> Result<ObjectId> {
        if row.is_empty() {
            let query_str = format!("INSERT INTO {} DEFAULT VALUES", schema.table_name);
            self.execute(query_str.as_str(), params![])?;
        } else {
            let res = self.execute(
                schema.make_insert_query_str().as_str(),
                params_from_iter(row.to_sql_row().into_iter()),
            );

            if res.is_err() {
                if let Some(error) =
                    get_missing_column_error(res.err().unwrap().to_string().as_str(), schema)
                {
                    return Err(MissingColumn(Box::new(error)));
                }
            }
        }

        Ok(ObjectId::from(self.last_insert_rowid()))
    }

    fn update_row(&self, id: ObjectId, schema: &Schema, row: &RowSlice) -> Result<()> {
        if schema.info.is_empty() {
            return Ok(());
        }
        let mut params = row.to_sql_row();
        params.push(id.into_sql());
        self.execute(
            schema.make_update_query_str().as_str(),
            params_from_iter(params.into_iter()),
        )?;
        Ok(())
    }

    fn select_row(&self, id: ObjectId, schema: &Schema) -> Result<Row<'static>> {
        let query = self.prepare(schema.make_select_query_str().as_str());
        let res = if let Ok(mut query_res) = query {
            query_res.query_row(params![id.into_i64()], |row| {
                let mut result_row = Vec::new();
                for i in 0..schema.info.len() {
                    result_row.push(match schema.info[i].data_type {
                        DataType::Int64 => Value::Int64(row.get(i)?),
                        DataType::Float64 => Value::Float64(row.get(i)?),
                        DataType::String => Value::String(Cow::Owned(row.get(i)?)),
                        DataType::Bytes => Value::Bytes(Cow::Owned(row.get(i)?)),
                        DataType::Bool => Value::Bool(row.get::<_, i64>(i)? > 0),
                    });
                }
                Ok(result_row)
            })
        } else {
            Err(query.err().unwrap())
        };

        if let Ok(x) = res {
            return Ok(x);
        }

        match res.err().unwrap() {
            rusqlite::Error::InvalidColumnType(i, _name, type_n) => {
                return Err(UnexpectedType(Box::new(UnexpectedTypeError::new(
                    schema.type_name,
                    schema.info[i].data_name,
                    schema.table_name,
                    schema.info[i].column_name,
                    schema.info[i].data_type,
                    type_n.to_string(),
                ))));
            }
            rusqlite::Error::SqliteFailure(_e, text) => {
                return Err(MissingColumn(Box::new(
                    get_missing_column_error(text.unwrap().as_str(), schema).unwrap(),
                )));
            }
            _ => {}
        };
        Err(NotFound(Box::new(NotFoundError::new(id, schema.type_name))))
    }

    fn delete_row(&self, id: ObjectId, schema: &Schema) -> Result<()> {
        if self
            .execute(
                schema.make_delete_query_str().as_str(),
                params![id.into_i64()],
            )
            .is_err()
        {
            return Err(NotFound(Box::new(NotFoundError::new(id, schema.type_name))));
        }
        Ok(())
    }

    fn commit(&self) -> Result<()> {
        self.execute("COMMIT", params![])?;
        Ok(())
    }

    fn rollback(&self) -> Result<()> {
        self.execute("ROLLBACK", params![])?;
        Ok(())
    }
}
