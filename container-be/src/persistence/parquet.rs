use std::sync::Arc;

use log::warn;
use parquet::basic::Type as BasicType;
use parquet::basic::{LogicalType, Repetition};
use parquet::schema::types::Type;
use sqlx::{Executor, Pool};

use crate::error::MyError;

use sqlx::Column; // Import the Column trait

pub async fn generate_parquet_schema_from_table(
    pool: &Pool<sqlx::Postgres>,
    table_name: &str,
) -> Result<Arc<Type>, MyError> {
    // Describe the table structure using SQLx
    let describe_query = format!("SELECT * FROM {} LIMIT 0", table_name);
    let describe = pool.describe(&describe_query).await?;

    // Build the Parquet schema
    let mut fields = Vec::new();

    for column in describe.columns() {
        let field = match column.type_info().to_string().as_str() {
            "INT4" | "INT8" => Type::primitive_type_builder(column.name(), BasicType::INT64)
                .with_repetition(Repetition::REQUIRED)
                .build()?,
            "TEXT" | "VARCHAR" => {
                Type::primitive_type_builder(column.name(), BasicType::BYTE_ARRAY)
                    .with_repetition(Repetition::REQUIRED)
                    .with_converted_type(parquet::basic::ConvertedType::UTF8)
                    // .with_repetition(Repetition::OPTIONAL)
                    .build()?
            }
            "BOOL" => Type::primitive_type_builder(column.name(), BasicType::BOOLEAN)
                // .with_repetition(Repetition::OPTIONAL)
                .build()?,
            "FLOAT4" | "FLOAT8" => Type::primitive_type_builder(column.name(), BasicType::DOUBLE)
                // .with_repetition(Repetition::OPTIONAL)
                .build()?,
            "TIMESTAMP" => Type::primitive_type_builder(column.name(), BasicType::INT64)
                // .with_repetition(Repetition::OPTIONAL)
                .with_converted_type(parquet::basic::ConvertedType::TIMESTAMP_MICROS)
                .build()?,
            _ => {
                warn!(
                    "Unsupported column type: {}",
                    column.type_info().to_string()
                );
                continue;
            }
        };

        fields.push(Arc::new(field));
    }

    println!("Fields: {:?}", fields);

    let schema = Type::group_type_builder("schema")
        .with_fields(fields)
        .build()?;

    Ok(Arc::new(schema))
}

// pub fn write_optional_column<T>(
//     column_writer: &mut parquet::column::writer::ColumnWriter,
//     values: &[T],
//     definition_levels: &[i16],
// ) -> Result<(), parquet::errors::ParquetError>
// where
//     T: parquet::data_type::DataType,
// {
//     match column_writer {
//         parquet::column::writer::ColumnWriter::Int64ColumnWriter(ref mut typed_writer) => {
//             typed_writer.write_batch(values, Some(definition_levels), None)?;
//         }
//         parquet::column::writer::ColumnWriter::ByteArrayColumnWriter(ref mut typed_writer) => {
//             typed_writer.write_batch(values, Some(definition_levels), None)?;
//         }
//         parquet::column::writer::ColumnWriter::BooleanColumnWriter(ref mut typed_writer) => {
//             typed_writer.write_batch(values, Some(definition_levels), None)?;
//         }
//         parquet::column::writer::ColumnWriter::DoubleColumnWriter(ref mut typed_writer) => {
//             typed_writer.write_batch(values, Some(definition_levels), None)?;
//         }
//         _ => {
//             return Err(parquet::errors::ParquetError::General(
//                 "Unsupported column type".to_string(),
//             ));
//         }
//     }
//     Ok(())
// }
