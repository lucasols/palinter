use serde::Deserialize;
use serde_yaml::Value;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum SingleOrMultiple<T> {
    Multiple(Vec<T>),
    Single(T),
}

#[derive(Deserialize, Debug, Clone)]
pub struct ParsedFileConditions {
    pub has_extension: Option<SingleOrMultiple<String>>,
    pub has_name: Option<SingleOrMultiple<String>>,
    pub does_not_have_name: Option<SingleOrMultiple<String>>,

    #[serde(flatten)]
    pub wrong: HashMap<String, Value>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ParsedFolderConditions {
    pub has_name_case: Option<String>,

    #[serde(flatten)]
    pub wrong: HashMap<String, Value>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ParsedFileExpect {
    pub name_case_is: Option<String>,
    pub error_msg: Option<String>,
    pub extension_is: Option<SingleOrMultiple<String>>,

    #[serde(flatten)]
    pub wrong: HashMap<String, Value>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ParsedFolderExpect {
    pub name_case_is: Option<String>,
    pub error_msg: Option<String>,

    #[serde(flatten)]
    pub wrong: HashMap<String, Value>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ParsedAnyOr<T> {
    Conditions(T),
    Any(String),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ParsedRule {
    File {
        #[serde(rename = "if_file")]
        conditions: ParsedAnyOr<ParsedFileConditions>,
        expect: Option<Box<ParsedAnyOr<SingleOrMultiple<ParsedFileExpect>>>>,
        expect_one_of: Option<Vec<ParsedFileExpect>>,
        error_msg: Option<String>,
        non_recursive: Option<bool>,
    },
    Folder {
        #[serde(rename = "if_folder")]
        conditions: ParsedAnyOr<ParsedFolderConditions>,
        expect: Option<ParsedAnyOr<SingleOrMultiple<ParsedFolderExpect>>>,
        expect_one_of: Option<Vec<ParsedFolderExpect>>,
        error_msg: Option<String>,
        non_recursive: Option<bool>,
    },
    OneOf {
        #[serde(rename = "one_of")]
        rules: Vec<ParsedRule>,
        error_msg: Option<String>,
    },
    Block(String),

    Error(Value),
}

#[derive(Deserialize, Debug, Clone)]
pub struct CorrectParsedFolderConfig {
    pub has_files: Option<Vec<String>>,
    pub rules: Option<Vec<ParsedRule>>,

    #[serde(flatten)]
    pub folders: BTreeMap<String, ParsedFolderConfig>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ParsedFolderConfig {
    Ok(CorrectParsedFolderConfig),
    Error(Value),
}

pub type ParsedBlocks = Option<BTreeMap<String, SingleOrMultiple<ParsedRule>>>;

#[derive(Deserialize, Debug, Clone)]
pub struct ParsedConfig {
    pub blocks: ParsedBlocks,
    pub global_rules: Option<Vec<ParsedRule>>,
    pub to_have_files: Option<Vec<String>>,

    #[serde(rename = "./")]
    pub root_folder: ParsedFolderConfig,
}

pub enum ParseFrom {
    Yaml,
    Json,
}

pub fn parse_config_string(config: &String, from: ParseFrom) -> Result<ParsedConfig, String> {
    match from {
        ParseFrom::Yaml => match serde_yaml::from_str(config) {
            Ok(config) => Ok(config),
            Err(err) => Err(format!(
                "Error parsing config: {}\n---\n{}\n---\n",
                err, config
            )),
        },
        ParseFrom::Json => match serde_json::from_str(config) {
            Ok(config) => Ok(config),
            Err(err) => Err(format!("Error parsing config: {}", err)),
        },
    }
}

pub fn parse_config(config_path: &str) -> Result<ParsedConfig, String> {
    let config = std::fs::read_to_string(config_path).unwrap();

    let is_json = config_path.ends_with(".json");

    parse_config_string(
        &config,
        if is_json {
            ParseFrom::Json
        } else {
            ParseFrom::Yaml
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn parse_config_works() {
        let config = parse_config("./src/fixtures/config1.json");

        insta::assert_debug_snapshot!(config);
    }
}
