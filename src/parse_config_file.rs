use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum SingleOrMultiple<T> {
    Single(T),
    Multiple(Vec<T>),
}

#[derive(Deserialize, Debug)]
pub struct FileConditions {
    pub has_extension: Option<SingleOrMultiple<String>>,
    pub has_name: Option<SingleOrMultiple<String>>,
    pub does_not_have_name: Option<SingleOrMultiple<String>>,
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
pub enum AnyOr<T> {
    Conditions(T),
    Any(String),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Rule {
    File {
        #[serde(rename = "if_file")]
        conditions: AnyOr<FileConditions>,
        expect: SingleOrMultiple<FileExpect>,
        extension_is: Option<SingleOrMultiple<String>>,
        error_msg: Option<String>,
    },
    Folder {
        #[serde(rename = "if_folder")]
        conditions: AnyOr<FileConditions>,
        expect: SingleOrMultiple<FileExpect>,
        error_msg: Option<String>,
    },
    OneOf {
        #[serde(rename = "one_of")]
        rules: Vec<Rule>,
    },
    Block(String),

    #[serde(deserialize_with = "ignore_contents")]
    Error,
}

#[derive(Deserialize, Debug)]
struct FolderConfig {
    has_files: Option<Vec<String>>,
    rules: Option<Vec<Rule>>,

    #[serde(flatten)]
    folders: BTreeMap<String, FolderConfig>,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub blocks: Option<BTreeMap<String, Rule>>,
    pub global_rules: Option<Vec<Rule>>,
    pub to_have_files: Option<Vec<String>>,

    #[serde(flatten)]
    folders: BTreeMap<String, FolderConfig>,
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

// fn validate_parsed_config(config: &Config) -> bool {
//     todo!("validate config");

//     true
// }

pub fn parse_config_string(config: String) -> Config {
    serde_json::from_str(&config).expect("Failed to parse config file")
}

pub fn parse_config(config_path: &str) -> Config {
    let config = std::fs::read_to_string(config_path).unwrap();

    // TODO: return error instead of unwrap
    let parsed = parse_config_string(config);

    // TODO: validate_parsed_config(&parsed);

    parsed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config_works() {
        let config = parse_config("./src/fixtures/config1.json");

        insta::assert_debug_snapshot!(config);
    }
}
