use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use insta::assert_debug_snapshot;

// enum NameCases {
//     CamelCase,
//     SnakeCase,
//     KebabCase,
//     PascalCase,
//     ConstantCase,
// }

// enum VecOr<T> {
//     Vec(Vec<T>),
//     T(T),
// }

// enum AnyOrRules<T> {
//     Any,
//     Rules(T),
// }

// struct FileConditions {
//     has_extension: Option<VecOr<String>>,
//     has_name: Option<VecOr<String>>,
//     has_name_case: Option<VecOr<NameCases>>,
//     does_not_have_name: Option<VecOr<String>>,
// }

// struct RootFilesConditions {
//     does_not_have_duplicate_name: Option<String>,
// }

// struct FolderConditions {
//     has_name: Option<VecOr<String>>,
//     has_name_case: Option<VecOr<NameCases>>,
//     root_files: Option<VecOr<RootFilesConditions>>,
// }

// struct FileExpect {
//     name_case_is: Option<VecOr<NameCases>>,
//     has_sibling_file: Option<VecOr<String>>,
//     extension_is: Option<VecOr<String>>,
//     has_parent_folder: Option<VecOr<String>>,
//     content_matches: Option<VecOr<String>>,
//     error_msg: Option<String>,
//     name_not_matches: Option<VecOr<String>>,
// }

// struct RootFilesExpect {
//     has: Option<VecOr<String>>,
//     does_not_have: Option<VecOr<String>>,
// }

// struct FolderExpect {
//     name_case_is: Option<VecOr<NameCases>>,
//     name_matches: Option<VecOr<String>>,
//     root_files: Option<RootFilesExpect>,
//     error_msg: Option<String>,
// }

// enum RuleTypeOrBlock {
//     RuleType(RuleType),
//     Block(String),
// }

// enum RuleType {
//     File {
//         conditions: AnyOrRules<FileConditions>,
//         expect: VecOr<FileExpect>,
//     },
//     Folder {
//         conditions: AnyOrRules<FolderConditions>,
//         expect: VecOr<FolderExpect>,
//     },
//     OneOf(Vec<RuleTypeOrBlock>),
// }

// struct Rule {
//     non_recursive: Option<bool>,
//     description: Option<String>,
//     error_msg: Option<String>,
//     rule: RuleType,
// }

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum SingleOrMultiple<T> {
    Single(T),
    Multiple(Vec<T>),
}

#[derive(Deserialize, Debug)]
struct FileConditions {
    has_extension: Option<SingleOrMultiple<String>>,
    has_name: Option<SingleOrMultiple<String>>,
}

#[derive(Deserialize, Debug)]
struct FolderConditions {
    has_name: Option<SingleOrMultiple<String>>,
}

#[derive(Deserialize, Debug)]
struct FileExpect {
    name_case_is: Option<SingleOrMultiple<String>>,
    has_sibling_file: Option<SingleOrMultiple<String>>,
    content_matches_any: Option<SingleOrMultiple<String>>,
    error_msg: Option<String>,
    name_not_matches: Option<SingleOrMultiple<String>>,
    extension_is: Option<SingleOrMultiple<String>>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Condition {
    File {
        #[serde(rename = "if_file")]
        conditions: Option<FileConditions>,
        expect: SingleOrMultiple<FileExpect>,
    },
    Folder {
        #[serde(rename = "if_folder")]
        conditions: Option<FileConditions>,
        expect: SingleOrMultiple<FileExpect>,
    },
    OneOf {
        #[serde(rename = "one_of")]
        rules: Vec<Condition>,
    },

    #[serde(deserialize_with = "ignore_contents")]
    Error,
}

#[derive(Deserialize, Debug)]
struct FolderConfig {
    has_files: Option<Vec<String>>,
    rules: Option<Vec<Condition>>,

    #[serde(flatten)]
    folders: HashMap<String, FolderConfig>,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    blocks: Option<HashMap<String, Condition>>,
    global_rules: Option<Vec<Condition>>,
    to_have_files: Option<Vec<String>>,

    #[serde(flatten)]
    folders: HashMap<String, FolderConfig>,
}

fn ignore_contents<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    #[allow(unused_must_use)]
    {
        // Ignore any content at this part of the json structure
        deserializer.deserialize_ignored_any(serde::de::IgnoredAny);
    }

    // Return unit as our 'Unknown' variant has no args
    Ok(())
}

pub fn parse_config(config_path: &str) -> Config {
    let config = std::fs::read_to_string(config_path).unwrap();
    let parsed: Config = serde_json::from_str(&config).unwrap();
    parsed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config_works() {
        let config = parse_config("./src/fixtures/config1.json");

        assert_debug_snapshot!(config);

        println!("{:#?}", config);
    }
}
