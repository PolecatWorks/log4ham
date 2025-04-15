use std::sync::Arc;

use log::{debug, warn};
use parquet::basic::Type as BasicType;

use parquet::basic::{LogicalType, Repetition, TimeUnit, Type as PhysicalType};
use parquet::schema::types::Type;
use sqlx::postgres::PgTypeInfo;
use sqlx::TypeInfo;
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
        let field_type = column.type_info();

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

    debug!("Fields: {:?}", fields);

    let schema = Type::group_type_builder("schema")
        .with_fields(fields)
        .build()?;

    Ok(Arc::new(schema))
}

// Helper function to map PgTypeInfo to Parquet type details
fn map_pg_type_to_parquet(
    col_name: &str,
    pg_type_info: &PgTypeInfo,
) -> Result<(PhysicalType, Option<LogicalType>), MyError> {
    let type_name = pg_type_info.name();
    debug!("Mapping PG column '{}' with type '{}'", col_name, type_name);

    match type_name {
        // Integers
        "INT2" | "SMALLINT" | "SMALLSERIAL" | "INT4" | "INTEGER" | "SERIAL" => {
            Ok((PhysicalType::INT32, None))
        }
        "INT8" | "BIGINT" | "BIGSERIAL" | "OID" => Ok((PhysicalType::INT64, None)),

        // Floats
        "FLOAT4" | "REAL" => Ok((PhysicalType::FLOAT, None)),
        "FLOAT8" | "DOUBLE PRECISION" => Ok((PhysicalType::DOUBLE, None)),

        // Boolean
        "BOOL" | "BOOLEAN" => Ok((PhysicalType::BOOLEAN, None)),

        // Text / String types
        "VARCHAR" | "TEXT" | "CHAR" | "BPCHAR" | "NAME" | "UNKNOWN" => {
            Ok((PhysicalType::BYTE_ARRAY, Some(LogicalType::String))) // formerly UTF8
        }

        // Binary data
        "BYTEA" => Ok((PhysicalType::BYTE_ARRAY, None)),

        // Date/Time types
        "TIMESTAMP" => Ok((
            // Timestamp without timezone
            PhysicalType::INT64,
            Some(LogicalType::Timestamp {
                is_adjusted_to_u_t_c: false,                // Not explicitly UTC
                unit: TimeUnit::MICROS(Default::default()), // Store microseconds
            }),
        )),
        "TIMESTAMPTZ" => Ok((
            // Timestamp with timezone (implicitly UTC in Rust/Parquet)
            PhysicalType::INT64,
            Some(LogicalType::Timestamp {
                is_adjusted_to_u_t_c: true,
                unit: TimeUnit::MICROS(Default::default()), // Store microseconds
            }),
        )),
        "DATE" => Ok((PhysicalType::INT32, Some(LogicalType::Date))),
        // Consider mapping TIME to INT64 MICROS if needed, requires LogicalType::Time
        "TIME" => Ok((
            PhysicalType::INT64,
            Some(LogicalType::Time {
                is_adjusted_to_u_t_c: false, // PostgreSQL TIME doesn't store timezone
                unit: TimeUnit::MICROS(Default::default()),
            }),
        )),

        // UUID
        "UUID" => Ok((PhysicalType::FIXED_LEN_BYTE_ARRAY, Some(LogicalType::Uuid))),
        // JSON
        "JSON" | "JSONB" => Ok((PhysicalType::BYTE_ARRAY, Some(LogicalType::Json))),

        // --- Complex Types (Handling requires more sophisticated logic) ---
        // Decimal/Numeric: Mapping is tricky. BYTE_ARRAY (String) is safest for dynamic schema.
        // Scale/Precision needed for Decimal Logical Type aren't easily available here.
        "NUMERIC" | "DECIMAL" => {
            warn!(
                "Mapping NUMERIC/DECIMAL column '{}' to String (BYTE_ARRAY) for safety.",
                col_name
            );
            Ok((PhysicalType::BYTE_ARRAY, Some(LogicalType::String)))
        }

        // Arrays: Parquet requires LIST structure. Skipping in this basic example.
        name if name.starts_with('_') => {
            warn!(
                "Skipping array column '{}' (type: {}). Dynamic LIST generation not implemented.",
                col_name, name
            );
            Err(MyError::Message("Array column skipped")) // Signal to skip this column
        }

        // Fallback for other unknown types
        _ => {
            warn!(
                "Unknown PostgreSQL type '{}' for column '{}'. Falling back to String (BYTE_ARRAY).",
                type_name, col_name
            );
            Ok((PhysicalType::BYTE_ARRAY, Some(LogicalType::String)))
        }
    }
}
