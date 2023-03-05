use std::collections::HashMap;

use crate::parse_config_file::{
    ParsedAnyOr, ParsedConfig, ParsedFolderConfig, ParsedRule, SingleOrMultiple,
};

#[derive(Debug)]
pub enum AnyOr<T> {
    Any,
    Or(T),
}

#[derive(Debug)]
pub enum NameCase {
    CamelCase,
    SnakeCase,
    KebabCase,
    PascalCase,
    ConstantCase,
}

#[derive(Debug)]
pub struct FileExpect {
    pub name_case_is: Option<NameCase>,
}

#[derive(Debug)]
pub struct FileConditions {
    pub has_extension: Option<Vec<String>>,
    // has_name: Option<Vec<String>>,
    // does_not_have_name: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct FileRule {
    pub conditions: AnyOr<FileConditions>,
    pub expect: AnyOr<Vec<FileExpect>>,
    // TODO: id: Option<String>,
}

pub struct Folder {
    pub name: String,
    pub rules: Vec<FileRule>,
}

#[derive(Debug)]
pub struct FolderConfig {
    pub sub_folders_config: HashMap<String, FolderConfig>,
    pub file_rules: Vec<FileRule>,
}

#[derive(Debug)]
pub struct Config {
    pub global_files_rules: Vec<FileRule>,
    pub root_folder: FolderConfig,
}

fn normalize_single_or_multiple<T: Clone>(single_or_multiple: &SingleOrMultiple<T>) -> Vec<T> {
    match single_or_multiple {
        SingleOrMultiple::Single(single) => vec![single.clone()],
        SingleOrMultiple::Multiple(multiple) => multiple.to_vec(),
    }
}

pub fn normalize_single_or_multiple_some<T: Clone>(
    single_or_multiple_option: &Option<SingleOrMultiple<T>>,
) -> Option<Vec<T>> {
    single_or_multiple_option
        .as_ref()
        .map(|single_or_multiple| normalize_single_or_multiple(single_or_multiple))
}

fn check_any(any: &String, config_path: &String) {
    if any != "any" {
        panic!("Invalid any: '{}' in '{}'", any, config_path);
    }
}

fn normalize_name_case(name_case: &String, config_path: &String) -> NameCase {
    match name_case.as_str() {
        "camelCase" => NameCase::CamelCase,
        "snake_case" => NameCase::SnakeCase,
        "kebab-case" => NameCase::KebabCase,
        "PascalCase" => NameCase::PascalCase,
        "CONSTANT_CASE" => NameCase::ConstantCase,
        _ => panic!("Invalid name case: '{}' in '{}'", name_case, config_path),
    }
}

fn normalize_rules(rules: &Vec<ParsedRule>, config_path: &String) -> Vec<FileRule> {
    let mut file_rules: Vec<FileRule> = vec![];

    for rule in rules {
        match rule {
            ParsedRule::File {
                conditions, expect, ..
            } => {
                let conditions = match conditions {
                    ParsedAnyOr::Any(any) => {
                        check_any(any, config_path);
                        AnyOr::Any
                    }
                    ParsedAnyOr::Conditions(conditions) => AnyOr::Or(FileConditions {
                        has_extension: normalize_single_or_multiple_some(&conditions.has_extension),
                    }),
                };

                let new_expect = match expect {
                    ParsedAnyOr::Any(any) => {
                        check_any(any, config_path);
                        AnyOr::Any
                    }
                    ParsedAnyOr::Conditions(expect_conditions) => {
                        let expects = normalize_single_or_multiple(expect_conditions)
                            .iter()
                            .map(|parsed_expect| FileExpect {
                                name_case_is: parsed_expect
                                    .name_case_is
                                    .as_ref()
                                    .map(|name_case| normalize_name_case(name_case, config_path)),
                            })
                            .collect();

                        AnyOr::Or(expects)
                    }
                };

                file_rules.push(FileRule {
                    conditions,
                    expect: new_expect,
                });
            }
            ParsedRule::Folder { .. } => {
                todo!("Error: Folder rules are not supported yet");
            }
            _ => {
                todo!("Error: Invalid rule");
            }
        }
    }

    file_rules
}

fn normalize_folder_config(
    folder_config: &ParsedFolderConfig,
    folder_path: String,
) -> Result<FolderConfig, String> {
    match folder_config {
        ParsedFolderConfig::Error => {
            Err(format!("Error: Invalid folder config in '{}'", folder_path))
        }
        ParsedFolderConfig::Ok(config) => {
            let file_rules = match &config.rules {
                Some(files) => normalize_rules(files, &folder_path),
                None => vec![],
            };

            let mut sub_folders_config: HashMap<String, FolderConfig> = HashMap::new();

            for (sub_folder_name, sub_folder_config) in &config.folders {
                let folder_path = format!("{}{}", folder_path, sub_folder_name);

                sub_folders_config.insert(
                    sub_folder_name.clone(),
                    normalize_folder_config(sub_folder_config, folder_path)?,
                );
            }

            Ok(FolderConfig {
                file_rules,
                sub_folders_config,
            })
        }
    }
}

pub fn get_config(parsed_config: &ParsedConfig) -> Result<Config, String> {
    let global_files_rules = match &parsed_config.global_rules {
        Some(global_rules) => normalize_rules(global_rules, &String::from("global_config")),
        None => vec![],
    };

    Ok(Config {
        global_files_rules,
        root_folder: normalize_folder_config(&parsed_config.root_folder, String::from("."))?,
    })
}
