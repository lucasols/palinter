use crate::internal_config::NameCase;
use regex::Regex;

pub fn name_case_is(name: &str, name_case_is: &NameCase) -> Result<(), String> {
    match name_case_is {
        NameCase::KebabCase => {
            let kebab_case_regex = Regex::new(r"^[a-z][a-z0-9-]+$").unwrap();

            if !kebab_case_regex.is_match(name) {
                return Err("should be named in kebab-case".to_string());
            }
        }
        NameCase::CamelCase => {
            let camel_case_regex = Regex::new(r"^[a-z][a-zA-Z0-9]+$").unwrap();

            if !camel_case_regex.is_match(name) {
                return Err("should be named in camelCase".to_string());
            }
        }
        NameCase::SnakeCase => {
            let snake_case_regex = Regex::new(r"^[a-z][a-z0-9_]+$").unwrap();

            if !snake_case_regex.is_match(name) {
                return Err("should be named in snake_case".to_string());
            }
        }
        NameCase::PascalCase => {
            let pascal_case_regex = Regex::new(r"^[A-Z][a-zA-Z0-9]+$").unwrap();

            if !pascal_case_regex.is_match(name) {
                return Err("should be named in PascalCase".to_string());
            }
        }
        NameCase::ConstantCase => {
            let constant_case_regex = Regex::new(r"^[A-Z][A-Z0-9_]+$").unwrap();

            if !constant_case_regex.is_match(name) {
                return Err("should be named in CONSTANT_CASE".to_string());
            }
        }
    }

    Ok(())
}
