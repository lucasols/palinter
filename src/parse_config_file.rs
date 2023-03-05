use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum SingleOrMultiple<T> {
    Single(T),
    Multiple(Vec<T>),
}

#[derive(Deserialize, Debug)]
pub struct ParsedFileConditions {
    pub has_extension: Option<SingleOrMultiple<String>>,
    pub has_name: Option<SingleOrMultiple<String>>,
    pub does_not_have_name: Option<SingleOrMultiple<String>>,
}

#[derive(Deserialize, Debug)]
struct FolderConditions {
    has_name: Option<SingleOrMultiple<String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ParsedFileExpect {
    pub name_case_is: Option<String>,
    has_sibling_file: Option<SingleOrMultiple<String>>,
    content_matches_any: Option<SingleOrMultiple<String>>,
    error_msg: Option<String>,
    name_not_matches: Option<SingleOrMultiple<String>>,
    extension_is: Option<SingleOrMultiple<String>>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ParsedAnyOr<T> {
    Conditions(T),
    Any(String),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ParsedRule {
    File {
        #[serde(rename = "if_file")]
        conditions: ParsedAnyOr<ParsedFileConditions>,
        expect: ParsedAnyOr<SingleOrMultiple<ParsedFileExpect>>,
        error_msg: Option<String>,
    },
    Folder {
        #[serde(rename = "if_folder")]
        conditions: ParsedAnyOr<ParsedFileConditions>,
        expect: ParsedAnyOr<SingleOrMultiple<ParsedFileExpect>>,
        error_msg: Option<String>,
    },
    OneOf {
        #[serde(rename = "one_of")]
        rules: Vec<ParsedRule>,
    },
    Block(String),

    #[serde(deserialize_with = "ignore_contents")]
    Error,
}

#[derive(Deserialize, Debug)]
pub struct CorrectParsedFolderConfig {
    pub has_files: Option<Vec<String>>,
    pub rules: Option<Vec<ParsedRule>>,

    #[serde(flatten)]
    pub folders: BTreeMap<String, ParsedFolderConfig>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ParsedFolderConfig {
    Ok(CorrectParsedFolderConfig),

    #[serde(deserialize_with = "ignore_contents")]
    Error,
}

#[derive(Deserialize, Debug)]
pub struct ParsedConfig {
    pub blocks: Option<BTreeMap<String, ParsedRule>>,
    pub global_rules: Option<Vec<ParsedRule>>,
    pub to_have_files: Option<Vec<String>>,

    #[serde(rename = "./")]
    pub root_folder: ParsedFolderConfig,
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

pub enum ParseFrom {
    Yaml,
    Json,
}

pub fn parse_config_string(config: String, from: ParseFrom) -> Result<ParsedConfig, String> {
    match from {
        ParseFrom::Yaml => match serde_yaml::from_str(&config) {
            Ok(config) => Ok(config),
            Err(err) => Err(format!(
                "Error parsing config: {}\n---\n{}\n---\n",
                err, config
            )),
        },
        ParseFrom::Json => match serde_json::from_str(&config) {
            Ok(config) => Ok(config),
            Err(err) => Err(format!("Error parsing config: {}", err)),
        },
    }
}

pub fn parse_config(config_path: &str) -> Result<ParsedConfig, String> {
    let config = std::fs::read_to_string(config_path).unwrap();

    let is_json = config_path.ends_with(".json");

    parse_config_string(
        config,
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
    fn parse_config_works() {
        let config = parse_config("./src/fixtures/config1.json");

        insta::assert_debug_snapshot!(config);
    }
}
