use crate::internal_config::NameCase;
use regex::Regex;

pub fn name_case_is(name: &str, name_case_is: &NameCase) -> Result<(), String> {
    match name_case_is {
        NameCase::KebabCase => {
            let kebab_case_regex = Regex::new(r"^[a-z][a-z0-9-]+$").unwrap();

            if !kebab_case_regex.is_match(name) {
                return Err(format!("File '{}' should be named in kebab-case", name));
            }
        }
        NameCase::CamelCase => {
            let camel_case_regex = Regex::new(r"^[a-z][a-zA-Z0-9]+$").unwrap();

            if !camel_case_regex.is_match(name) {
                return Err(format!("File '{}' should be named in camelCase", name));
            }
        }
        NameCase::SnakeCase => {
            let snake_case_regex = Regex::new(r"^[a-z][a-z0-9_]+$").unwrap();

            if !snake_case_regex.is_match(name) {
                return Err(format!("File '{}' should be named in snake_case", name));
            }
        }
        NameCase::PascalCase => {
            let pascal_case_regex = Regex::new(r"^[A-Z][a-zA-Z0-9]+$").unwrap();

            if !pascal_case_regex.is_match(name) {
                return Err(format!("File '{}' should be named in PascalCase", name));
            }
        }
        NameCase::ConstantCase => {
            let constant_case_regex = Regex::new(r"^[A-Z][A-Z0-9_]+$").unwrap();

            if !constant_case_regex.is_match(name) {
                return Err(format!("File '{}' should be named in CONSTANT_CASE", name));
            }
        }
    }

    Ok(())
}
