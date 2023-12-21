//! Implements parameter validation for LSP-servers.
//! 
//! 
//! 

use serde::{Serialize, Deserialize};
use serde_json::{Map, Value};

pub enum ParamValidationError {
    IsNotPassByArgument,
    Unrecognized(Unrecognized),
    InvalidParam(InvalidParam)
}

pub struct Unrecognized {

    unrecognized : Vec<String>
}

pub struct InvalidParam {
    property : String,
    message : String
}

pub trait ExpectedFields {
    fn expected_fields() -> Vec<String>;
}

pub fn from_value<'de, T : Deserialize<'de> + ExpectedFields>(value : serde_json::Value) -> Result<T, ParamValidationError> {

    let value = match value {
        Value::Object(_) => value,
        Value::Null => Value::Null,
        _ => return Err(ParamValidationError::IsNotPassByArgument)
    };

    // Find unreconginsed arguments
    let expected_fields = T::expected_fields();
    let expected_fields : Vec<&str> = expected_fields.iter().map(|x| x.as_ref()).collect();
    let unrecognized = list_unrecogninzed_fields(expected_fields.as_ref(), &value);
    if ! unrecognized.is_empty() {
        return Err(ParamValidationError::Unrecognized(Unrecognized { unrecognized}))
    }

    serde_path_to_error::deserialize::<'de, _, T>(value)
        .map_err(|e| ParamValidationError::InvalidParam(
            InvalidParam {
                property : e.path().to_string(),
                message : e.to_string()
            }
        ))
}

fn list_unrecogninzed_fields(
    expected_arguments : &[&str],
    json_value : &serde_json::Value
) -> Vec<String> {

    match json_value {
        Value::Object(map) => {
            let mut current_prefix = Vec::new();
            let mut current_result = Vec::new();
            list_unrecognized_fields_impl(
                expected_arguments, 
                &map,
                &mut current_prefix,
                &mut current_result
            )
        },
        _ => {
            Vec::new()
        }
    }
}

fn list_unrecognized_fields_impl(
    expected_arguments : &[&str],
    map : &Map<String, Value>, 
    current_prefix : &mut Vec<Box<str>>, 
    current_result : &mut Vec<String>
    ) -> Vec<String> {

   
    for (name, value) in map {
        match value {
            Value::Object(map) => {
                current_prefix.push(name.clone().into_boxed_str());
                list_unrecognized_fields_impl(
                    expected_arguments,
                    map,
                    current_prefix,
                    current_result
                );
                current_prefix.pop();
            }
            _ => {
                current_prefix.push(name.clone().into_boxed_str());
                let argument = current_prefix.join(".");
                if ! expected_arguments.contains(&argument.as_ref()) {
                    current_result.push(argument.to_string());
                }
                current_prefix.pop();
            }
       };
    }

    current_result.clone()
}

#[cfg(test)]
mod test {
    use super::list_unrecogninzed_fields;
    use serde_json::json;

    #[test]
    fn test_find_unrecognized_arguments() {
        struct Case<'a> {
            value : serde_json::Value,
            expected_arguments : &'a[&'a str],
            expected_result : Vec<String>
        };


        let cases = vec![
            Case {
                value : serde_json::json!({}),
                expected_arguments : &[],
                expected_result : vec![]
            },
            Case {
                value : serde_json::json!({"param_a": "a"}),
                expected_arguments : &[],
                expected_result : vec!["param_a".to_string()]
            },
            Case {
                value : json!({"param_a" : "a"}),
                expected_arguments : &["param_a"],
                expected_result : vec![]
            },
            Case {
                value : serde_json::json!({"param_a" : {"field_a" : "a"}}),
                expected_arguments : &[],
                expected_result : vec!["param_a.field_a".to_string()]
            },
            Case {
                value : serde_json::json!({"param_a" : {"field_a" : "a"}}),
                expected_arguments : &["param_a.field_a"],
                expected_result : vec![]
            },
            Case {
                value : serde_json::json!({"param_a" : "a", "param_b" : "b"}),
                expected_arguments : &["f1", "f2"],
                expected_result : vec!["param_a".to_string(), "param_b".to_string()]
            }
 
        ];

        for case in cases {
            let result = list_unrecogninzed_fields(
                case.expected_arguments,
                 &case.value);
            assert_eq!(result, case.expected_result);

        }
    }
}