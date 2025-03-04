//! Module to describe schema management functions
//!
//! Provide functions to create, generate and validate schemas

use std::{
    fs::File,
    io::{BufReader, Read},
};

use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::MyError;

/// Describe a person using a simple data structure
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct Person {
    /// Name of the Person
    pub name: String,
    /// Persons age
    pub age: u32,
}

/// Create and return the JsonSchema for the [Person] object
pub fn schema_person_string() -> Result<String, MyError> {
    let my_schema = schema_for!(Person);
    let my_schema_json = serde_json::to_string_pretty(&my_schema)?;
    let my_schema_string = my_schema_json;
    // println!("{my_schema_string}");
    Ok(my_schema_string)
}

/// Return a JsonSchema of a vector of [Person]
pub fn stdout_schema_list() -> Result<(), MyError> {
    let my_schema_list = schema_for!(Vec<Person>);
    let my_schema_list_json = serde_json::to_string_pretty(&my_schema_list)?;
    println!("{my_schema_list_json}");
    Ok(())
}

/// Dynamic generate of JsonSchema
///
/// Can use this to generate the JsonSchema for any [Serialisable] object rather than writing one function for each object.
/// This function can be used instead of [stdout_schema_list] and [schema_person_string]
pub fn schema_string<MyType: JsonSchema>() -> Result<String, MyError> {
    let my_schema = schema_for!(MyType);
    let my_schema_json = serde_json::to_string_pretty(&my_schema)?;
    let my_schema_string = my_schema_json;
    Ok(my_schema_string)
}

/// Write a JSON file with with [count] copies of the [Person] object
///
/// * `filename` - A string slice that naem fo the file to generate
/// * `count` - a unsigned int to define how many copies of [Person] to generate
pub fn write_records(filename: &str, count: u32) -> Result<(), MyError> {
    let mydata: Vec<_> = (0..count)
        .map(|x| Person {
            name: format!("name-{:08}", x),
            age: x,
        })
        .collect();
    let file = File::create(filename)?;

    serde_json::to_writer(file, &mydata)?;

    Ok(())
}



// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn validate_vec() {
//         let schema = schema_string::<Vec<Person>>().expect("got schema");

//         println!("my schema is {schema}");

//         let error_limit = 2;

//         let validated = if false {
//             let file = File::open("myfile.json").expect("open file");
//             let reader = BufReader::new(file);
//             validate(&schema, reader, error_limit).expect("validated")
//         } else if true {
//             let size = 220;

//             let my_data_vec: Vec<_> = (0..size)
//                 .map(|x| Person {
//                     name: format!("name-{:08}", x),
//                     age: x,
//                 })
//                 .collect();
//             let my_data_string = serde_json::to_string_pretty(&my_data_vec).expect("to string");
//             let my_data_bytes = my_data_string.as_bytes();
//             validate(&schema, my_data_bytes, error_limit).expect("validated")
//         } else {
//             let inline_example_array = br#"[{"name":"name-00000000","age":0},{"name":"name-00000001","age":1},{"name":"name-00000002","age":2}]"#;

//             let inline_example_slice = &inline_example_array[..]; // Has into for reader (https://doc.rust-lang.org/stable/std/io/trait.BufRead.html)
//             validate(&schema, inline_example_slice, error_limit).expect("validated")
//         };

//         let validated_typed: Vec<Person> =
//             serde_json::from_value(validated).expect("conversion to Explicit Type");

//         println!("There are {} records", validated_typed.len());
//     }
// }
