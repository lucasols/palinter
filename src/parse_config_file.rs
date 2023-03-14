use serde::Deserialize;
use serde_yaml::Value;
use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum SingleOrMultiple<T> {
    Multiple(Vec<T>),
    Single(T),
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct ParsedFileConditions {
    pub has_extension: Option<SingleOrMultiple<String>>,
    pub has_name: Option<String>,
    pub not_has_name: Option<String>,

    #[serde(flatten)]
    pub wrong: HashMap<String, Value>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ParsedFileExpect {
    pub name_case_is: Option<String>,
    pub extension_is: Option<SingleOrMultiple<String>>,
    pub has_sibling_file: Option<String>,
    pub content_matches: Option<ParsedFileContentMatches>,
    pub content_matches_any: Option<ParsedFileContentMatches>,
    pub name_is: Option<String>,
    pub name_is_not: Option<String>,

    pub error_msg: Option<String>,

    #[serde(flatten)]
    pub wrong: HashMap<String, Value>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ParsedFolderConditions {
    pub has_name_case: Option<String>,
    pub has_name: Option<String>,
    pub not_has_name: Option<String>,
    pub root_files_find_pattern: Option<ParsedFindPattern>,

    #[serde(flatten)]
    pub wrong: HashMap<String, Value>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ParsedFolderExpect {
    pub name_case_is: Option<String>,
    pub name_is: Option<String>,
    pub name_is_not: Option<String>,
    pub root_files_has: Option<String>,
    pub root_files_has_not: Option<String>,

    pub error_msg: Option<String>,

    #[serde(flatten)]
    pub wrong: HashMap<String, Value>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ParsedFileContentMatchesConfig {
    pub all: Option<Vec<String>>,
    pub any: Option<Vec<String>>,
    pub at_least: Option<usize>,
    pub at_most: Option<usize>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ParsedFileContentMatchesItem {
    Single(String),
    Config(ParsedFileContentMatchesConfig),

    Error(Value),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ParsedFileContentMatches {
    Single(String),
    Multiple(Vec<ParsedFileContentMatchesItem>),

    Error(Value),
}

#[derive(Deserialize, Debug, Clone)]
pub struct ParsedFindPattern {
    pub pattern: String,
    pub at_least: Option<usize>,
    pub at_most: Option<usize>,
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
        not_touch: Option<bool>,
    },
    Folder {
        #[serde(rename = "if_folder")]
        conditions: ParsedAnyOr<ParsedFolderConditions>,
        expect: Option<Box<ParsedAnyOr<SingleOrMultiple<ParsedFolderExpect>>>>,
        expect_one_of: Option<Vec<ParsedFolderExpect>>,
        error_msg: Option<String>,
        non_recursive: Option<bool>,
        not_touch: Option<bool>,
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
    pub has_files_in_root: Option<Vec<String>>,
    pub rules: Option<Vec<ParsedRule>>,
    pub optional: Option<bool>,
    pub allow_unconfigured_files: Option<bool>,
    pub allow_unconfigured_folders: Option<bool>,

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
    pub to_have_files: Option<Vec<String>>,
    pub analyze_content_of_files_types: Option<Vec<String>>,
    pub ignore: Option<Vec<String>>,

    #[serde(rename = "./")]
    pub root_folder: ParsedFolderConfig,

    #[serde(flatten)]
    pub wrong: HashMap<String, Value>,
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

pub fn parse_config_file(config_path: &PathBuf) -> Result<ParsedConfig, String> {
    let config = std::fs::read_to_string(config_path).map_err(|err| err.to_string())?;

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