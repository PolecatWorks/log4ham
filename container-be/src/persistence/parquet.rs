use std::sync::Arc;

use log::{debug, info, warn};
use parquet::basic::{LogicalType, Repetition, TimeUnit};
use parquet::basic::Type as BasicType;
use parquet::schema::types::Type;
use sqlx::Pool;

use crate::error::MyError;



use sqlx::postgres::{PgColumn, PgTypeInfo};
use sqlx::Column; // Import the Column trait


// Helper function to map PgTypeInfo to Parquet type details
fn map_pg_type_to_parquet(
    col_name: &str,
    pg_type_info: &PgTypeInfo,
) -> Result<(BasicType, Option<LogicalType>), MyError> {
    let type_name = format!("{:?}", pg_type_info);
    debug!("Mapping PG column '{}' with type '{}'", col_name, type_name);

    match type_name {
        // Integers
        "INT2" | "SMALLINT" | "SMALLSERIAL" | "INT4" | "INTEGER" | "SERIAL" => {
            Ok((BasicType::INT32, None))
        }
        "INT8" | "BIGINT" | "BIGSERIAL" | "OID" => Ok((BasicType::INT64, None)),

        // Floats
        "FLOAT4" | "REAL" => Ok((BasicType::FLOAT, None)),
        "FLOAT8" | "DOUBLE PRECISION" => Ok((BasicType::DOUBLE, None)),

        // Boolean
        "BOOL" | "BOOLEAN" => Ok((BasicType::BOOLEAN, None)),

        // Text / String types
        "VARCHAR" | "TEXT" | "CHAR" | "BPCHAR" | "NAME" | "UNKNOWN" => {
            Ok((BasicType::BYTE_ARRAY, Some(LogicalType::String))) // formerly UTF8
        }

        // Binary data
        "BYTEA" => Ok((BasicType::BYTE_ARRAY, None)),

        // Date/Time types
        "TIMESTAMP" => Ok(( // Timestamp without timezone
            BasicType::INT64,
            Some(LogicalType::Timestamp {
                is_adjusted_to_u_t_c: false, // Not explicitly UTC
                unit: TimeUnit::MICROS(Default::default()), // Store microseconds
            }),
        )),
         "TIMESTAMPTZ" => Ok(( // Timestamp with timezone (implicitly UTC in Rust/Parquet)
             BasicType::INT64,
             Some(LogicalType::Timestamp {
                is_adjusted_to_u_t_c: true,
                unit: TimeUnit::MICROS(Default::default()), // Store microseconds
            }),
         )),
        "DATE" => Ok((BasicType::INT32, Some(LogicalType::Date))),
        // Consider mapping TIME to INT64 MICROS if needed, requires LogicalType::Time
        "TIME" => Ok((
             BasicType::INT64,
             Some(LogicalType::Time {
                 is_adjusted_to_u_t_c: false, // PostgreSQL TIME doesn't store timezone
                 unit: TimeUnit::MICROS(Default::default()),
             }),
         )),


        // UUID
        "UUID" => Ok((
            BasicType::FIXED_LEN_BYTE_ARRAY,
            Some(LogicalType::Uuid),
        )),
        // JSON
        "JSON" | "JSONB" => Ok((BasicType::BYTE_ARRAY, Some(LogicalType::Json))),

        // --- Complex Types (Handling requires more sophisticated logic) ---
        // Decimal/Numeric: Mapping is tricky. BYTE_ARRAY (String) is safest for dynamic schema.
        // Scale/Precision needed for Decimal Logical Type aren't easily available here.
        "NUMERIC" | "DECIMAL" => {
            warn!("Mapping NUMERIC/DECIMAL column '{}' to String (BYTE_ARRAY) for safety.", col_name);
            Ok((BasicType::BYTE_ARRAY, Some(LogicalType::String)))
        }

        // Arrays: Parquet requires LIST structure. Skipping in this basic example.
        name if name.starts_with('_') => {
             warn!("Skipping array column '{}' (type: {}). Dynamic LIST generation not implemented.", col_name, name);
             Err(MyError::Message(&format!("Array column '{}' skipped", col_name))) // Signal to skip this column
         }

        // Fallback for other unknown types
        _ => {
            warn!(
                "Unknown PostgreSQL type '{}' for column '{}'. Falling back to String (BYTE_ARRAY).",
                type_name, col_name
            );
            Ok((BasicType::BYTE_ARRAY, Some(LogicalType::String)))
        }
    }
}


async fn generate_schema_from_query<DB: sqlx::Database>(
    sql: &str,
    pool: &Pool<DB>, // Adjust DB type if needed
) -> Result<Arc<Type>, MyError> {
    info!("Describing query to generate Parquet schema...");
    let describe_result = pool.describe(sql).await?; // Use describe()
    info!(
        "Query description received. Found {} columns.",
        describe_result.columns().len()
    );

    let mut fields: Vec<Arc<Type>> = Vec::new();

    for (i, column) in describe_result.columns().iter().enumerate() {
        let col_name = column.name();
        let type_info = column.type_info();

        // Determine repetition based on nullability info from describe()
        let nullable = describe_result
            .nullable(i) // Check nullability by index
            .unwrap_or(true); // Default to nullable (OPTIONAL) if unknown
        let repetition = if nullable {
            Repetition::OPTIONAL
        } else {
            Repetition::REQUIRED
        };

        // Map the PG type to Parquet physical/logical types
        match map_pg_type_to_parquet(col_name, type_info) {
             Ok((physical_type, logical_type_opt)) => {
                let mut builder = Type::primitive_type_builder(col_name, physical_type)
                    .with_repetition(repetition);

                if let Some(logical_type) = logical_type_opt {
                     // Special handling for UUID which needs length
                     if let LogicalType::Uuid = logical_type {
                         builder = builder.with_length(16); // UUID is 16 bytes
                     }
                    builder = builder.with_logical_type(Some(logical_type));
                }

                fields.push(Arc::new(builder.build()?));
                debug!(
                    "Added field '{}' ({:?}, {:?}, {:?})",
                    col_name, physical_type, logical_type_opt, repetition
                );
            }
             Err(e) => {
                 warn!("Skipping column '{}' due to mapping error: {}", col_name, e);
             }
        }
    }

    if fields.is_empty() {
        Err(MyError::Message("No valid fields found in the query description."));
    }

    // Build the final group type (message schema)
    let schema = Type::group_type_builder("schema") // You can customize the root name
        .with_fields(fields)
        .build()?;

    info!("Parquet schema generated successfully.");
    Ok(Arc::new(schema))
}
