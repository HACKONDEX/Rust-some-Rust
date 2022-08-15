#![forbid(unsafe_code)]
use proc_macro::{TokenStream};
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, Field, FieldsNamed, LitStr, Ident, Type, DataStruct};

fn parse_column_or_table_name (attributes: &[Attribute], table_or_column: &str) -> Option<String> {
    if attributes.is_empty() {
        return None;
    }
    if !attributes[0].path.is_ident(table_or_column) {
        return None;
    }
    let res: Result<LitStr, _> = attributes[0].parse_args();
    if res.is_err() {
        return None;
    }
    Some(res.unwrap().value())
}

fn get_fields_slice(data: Data) -> (Option<Vec<Field>>, bool) {
    let structure: Option<DataStruct> = match data {
        syn::Data::Struct(struc) => Some(struc),
        _ => {None},
    };

    let mut table_is_empty = false;
    let fields_opt =match structure {
        Some(struc) => {
            match struc.fields {
                Fields::Named(FieldsNamed{named, ..}) => Some(named.into_iter().collect::<Vec::<Field>>()),
                Fields::Unit => {table_is_empty = true; None},
                _ => {None},
            }
        },
        _ => {None},
    };
    (fields_opt, table_is_empty)
}

fn create_code(ident:Ident,
               table_name: String,
               fields_names: Vec::<Ident>,
               column_names: Vec::<String>,
               types_names: Vec::<Type>) -> TokenStream {
    let code = quote! {
        impl ::orm::Object for #ident {
            const SCHEMA: &'static ::orm::object::Schema = &::orm::object::Schema {
                table_name: #table_name,
                type_name: stringify!(#ident),
                info: &[#(::orm::object::ColumnInfo {
                    data_name: stringify!(#fields_names),
                    data_type: <#types_names as ::orm::data::ObjectColumnType>::NAME,
                    column_name: #column_names,
                    },)*],
            };

            fn get_row_from_object(&self) -> ::orm::storage::Row {
                let values = vec![#((&self.#fields_names).into()), *];
                let mut row: ::orm::storage::Row = Vec::new();
                for name in values{
                    row.push(name);
                }
                row
            }

            fn get_object_from_row(row: ::orm::storage::Row) -> Self {
                let mut iter = row.into_iter();
                Self { #(#fields_names: ::std::convert::From::from(iter.next().unwrap())), *}
            }
        }
    };
    code.into()
}

#[proc_macro_derive(Object, attributes(table_name, column_name))]
pub fn derive_object(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, attrs, ..
    } = parse_macro_input!(input);

    let table_name = match parse_column_or_table_name(&attrs, "table_name") {
        Some(name) => {name},
        None => {ident.to_string()},
    };

    // Columns Parse
    let (fields_vec_opt, table_is_empty) = get_fields_slice(data);
    if fields_vec_opt.is_none() && !table_is_empty {
        panic!("Something went wrong in #Derive(Object)!!! line 87");
    }
    let mut field_names = Vec::<Ident>::new();
    let mut column_names = Vec::<String>::new();
    let mut types_names = Vec::<Type>::new();
    if !table_is_empty {
        for field in fields_vec_opt.unwrap() {
            types_names.push(field.ty);
            column_names.push(match parse_column_or_table_name(&field.attrs, "column_name") {
                Some(x) => x,
                None => field.ident.as_ref().unwrap().to_string()
            });
            field_names.push(field.ident.unwrap());
        }
    }

    create_code(ident, table_name, field_names, column_names, types_names)
}

