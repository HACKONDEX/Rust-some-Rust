#![forbid(unsafe_code)]
use crate::{data::DataType, storage::Row};
use std::any::Any;

////////////////////////////////////////////////////////////////////////////////

pub struct Schema {
    pub type_name: &'static str,
    pub table_name: &'static str,
    pub info: &'static [ColumnInfo],
}

pub struct ColumnInfo {
    pub data_name: &'static str,
    pub data_type: DataType,
    pub column_name: &'static str,
}

impl Schema {
    pub fn make_create_query_str(&self) -> String {
        let mut common_part = format!(
            "CREATE TABLE {} (id INTEGER PRIMARY KEY AUTOINCREMENT",
            self.table_name
        );

        let mut end_part = "".to_string();

        for data in self.info {
            end_part.push_str(", ");
            end_part.push_str(data.column_name);
            end_part.push(' ');
            end_part.push_str(data.data_name);
        }
        common_part.push_str(end_part.as_str());
        common_part.push(')');
        common_part
    }

    pub fn make_insert_query_str(&self) -> String {
        let mut column_names = "".to_string();
        let mut values = "".to_string();
        for i in 0..self.info.len() {
            column_names.push_str(self.info[i].column_name);
            if i != self.info.len() - 1 {
                values.push_str("?,");
                column_names.push(',');
            } else {
                values.push('?');
            }
        }
        format!(
            "INSERT INTO {}({}) VALUES({})",
            self.table_name, column_names, values
        )
    }

    pub fn make_update_query_str(&self) -> String {
        let mut set = "".to_string();
        let ind = self.info.len() - 1;
        for i in 0..self.info.len() {
            set.push_str(self.info[i].column_name);
            set.push_str("=?");
            if i != ind {
                set.push(',');
            }
        }

        format!("UPDATE {} SET {} WHERE id = ?", self.table_name, set)
    }

    pub fn make_select_query_str(&self) -> String {
        let mut query_str = "SELECT ".to_string();
        if self.info.is_empty() {
            query_str.push_str("1 ");
        } else {
            let ind = self.info.len() - 1;
            for i in 0..self.info.len() {
                query_str.push_str(self.info[i].column_name);
                if i != ind {
                    query_str.push(',');
                }
            }
        }
        query_str.push_str(" FROM ");
        query_str.push_str(self.table_name);
        query_str.push_str(" WHERE id = ?");
        query_str
    }

    pub fn make_delete_query_str(&self) -> String {
        format!("DELETE FROM {} WHERE id = ?", self.table_name)
    }
}

pub trait Object: Any + Sized {
    const SCHEMA: &'static Schema;
    fn get_row_from_object(&self) -> Row;
    fn get_object_from_row(row: Row) -> Self;
}

////////////////////////////////////////////////////////////////////////////////

pub trait Store {
    fn get_schema(&self) -> &'static Schema;
    fn get_row_from_store(&self) -> Row;
    fn cast_to_any(&self) -> &dyn Any;
    fn cast_to_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Object> Store for T {
    fn get_schema(&self) -> &'static Schema {
        Self::SCHEMA
    }

    fn get_row_from_store(&self) -> Row {
        Object::get_row_from_object(self)
    }

    fn cast_to_any(&self) -> &dyn Any {
        self
    }

    fn cast_to_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
