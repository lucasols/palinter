use std::collections::{BTreeMap, HashMap};

use serde_yaml::Value;

use crate::parse_config_file::{
    CorrectParsedFolderConfig, ParsedAnyOr, ParsedConfig, ParsedFolderConfig, ParsedRule,
    SingleOrMultiple,
};

#[derive(Debug, Clone)]
pub enum AnyOr<T> {
    Any,
    Or(T),
}

#[derive(Debug, Clone)]
pub enum NameCase {
    CamelCase,
    SnakeCase,
    KebabCase,
    PascalCase,
    ConstantCase,
}

#[derive(Debug, Clone)]
pub struct FileExpect {
    pub name_case_is: Option<NameCase>,
}

#[derive(Debug, Clone)]
pub struct FolderExpect {
    pub name_case_is: Option<NameCase>,
}

#[derive(Debug, Clone)]
pub struct FileConditions {
    pub has_extension: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct FolderConditions {}

#[derive(Debug, Clone)]
pub struct FileRule {
    pub conditions: AnyOr<FileConditions>,
    pub expect: AnyOr<Vec<FileExpect>>,
    pub non_recursive: bool,
}

#[derive(Debug, Clone)]
pub struct FolderRule {
    pub conditions: AnyOr<FolderConditions>,
    pub expect: AnyOr<Vec<FolderExpect>>,
    pub non_recursive: bool,
}

pub struct Folder {
    pub name: String,
    pub rules: Vec<FileRule>,
}

#[derive(Debug)]
pub struct FolderConfig {
    pub sub_folders_config: HashMap<String, FolderConfig>,
    pub file_rules: Vec<FileRule>,
    pub folder_rules: Vec<FolderRule>,
}

#[derive(Debug)]
pub struct Config {
    pub global_files_rules: Vec<FileRule>,
    pub global_folders_rules: Vec<FolderRule>,
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

fn check_invalid_conditions(
    extra_conditions: &HashMap<String, Value>,
    condition_type: &str,
    config_path: &String,
) -> Result<(), String> {
    if !extra_conditions.is_empty() {
        return Err(format!(
            "Error: Invalid {} found in '{}' rule: {}",
            condition_type,
            config_path,
            extra_conditions
                .keys()
                .map(|key| key.as_str())
                .collect::<Vec<&str>>()
                .join(", "),
        ));
    }

    Ok(())
}

fn normalize_rules(
    rules: &Vec<ParsedRule>,
    config_path: &String,
) -> Result<(Vec<FileRule>, Vec<FolderRule>), String> {
    let mut file_rules: Vec<FileRule> = vec![];
    let mut folder_rules: Vec<FolderRule> = vec![];

    for rule in rules {
        match rule {
            ParsedRule::File {
                conditions,
                expect,
                non_recursive,
                error_msg,
            } => {
                let conditions = match conditions {
                    ParsedAnyOr::Any(any) => {
                        check_any(any, config_path);
                        AnyOr::Any
                    }
                    ParsedAnyOr::Conditions(conditions) => {
                        check_invalid_conditions(
                            &conditions.wrong,
                            "if_file condition",
                            config_path,
                        )?;

                        AnyOr::Or(FileConditions {
                            has_extension: normalize_single_or_multiple_some(
                                &conditions.has_extension,
                            ),
                        })
                    }
                };

                let new_expect = match &**expect {
                    ParsedAnyOr::Any(any) => {
                        check_any(any, config_path);
                        AnyOr::Any
                    }
                    ParsedAnyOr::Conditions(expect_conditions) => {
                        let mut expects: Vec<FileExpect> = Vec::new();

                        for parsed_expected in normalize_single_or_multiple(expect_conditions) {
                            check_invalid_conditions(
                                &parsed_expected.wrong,
                                "file expect condition",
                                config_path,
                            )?;

                            expects.push(FileExpect {
                                name_case_is: parsed_expected
                                    .name_case_is
                                    .as_ref()
                                    .map(|name_case| normalize_name_case(name_case, config_path)),
                            });
                        }

                        AnyOr::Or(expects)
                    }
                };

                file_rules.push(FileRule {
                    conditions,
                    expect: new_expect,
                    non_recursive: non_recursive.unwrap_or(false),
                });
            }
            ParsedRule::Folder {
                conditions,
                error_msg,
                expect,
                non_recursive,
            } => {
                let conditions = match conditions {
                    ParsedAnyOr::Any(any) => {
                        check_any(any, config_path);
                        AnyOr::Any
                    }
                    ParsedAnyOr::Conditions(conditions) => {
                        check_invalid_conditions(
                            &conditions.wrong,
                            "if_folder condition",
                            config_path,
                        )?;

                        AnyOr::Or(FolderConditions {})
                    }
                };

                let new_expect = match expect {
                    ParsedAnyOr::Any(any) => {
                        check_any(any, config_path);
                        AnyOr::Any
                    }
                    ParsedAnyOr::Conditions(expect_conditions) => {
                        let mut expects: Vec<FolderExpect> = Vec::new();

                        for parsed_expected in normalize_single_or_multiple(expect_conditions) {
                            check_invalid_conditions(
                                &parsed_expected.wrong,
                                "file expect condition",
                                config_path,
                            )?;

                            expects.push(FolderExpect {
                                name_case_is: parsed_expected
                                    .name_case_is
                                    .as_ref()
                                    .map(|name_case| normalize_name_case(name_case, config_path)),
                            });
                        }

                        AnyOr::Or(expects)
                    }
                };

                folder_rules.push(FolderRule {
                    conditions,
                    expect: new_expect,
                    non_recursive: non_recursive.unwrap_or(false),
                });
            }
            ParsedRule::Block(block) => {
                todo!("Block rules are not implemented yet.")
            }
            ParsedRule::OneOf { rules } => {
                todo!("OneOf rules are not implemented yet.")
            }
            ParsedRule::Error => {
                return Err(format!("Error: Invalid rule in '{}'", config_path));
            }
        }
    }

    Ok((file_rules, folder_rules))
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
            let (file_rules, folder_rules) = match &config.rules {
                Some(files) => normalize_rules(files, &folder_path)?,
                None => (vec![], vec![]),
            };

            let mut sub_folders_config: HashMap<String, FolderConfig> = HashMap::new();

            for (sub_folder_name, sub_folder_config) in &config.folders {
                if !sub_folder_name.starts_with('/') {
                    return Err(format!(
                        "Invalid sub folder name: '{}' in '{}', folders name should start with '/'",
                        sub_folder_name, folder_path
                    ));
                }

                let compound_path_parts = sub_folder_name.split('/').collect::<Vec<&str>>();

                if compound_path_parts.len() > 2 {
                    let fisrt_part = format!("/{}", compound_path_parts[1]);

                    if config.folders.contains_key(&fisrt_part) {
                        return Err(format!(
                            "Duplicate compound folder path: '{}' in '{}', compound folder paths should not conflict with existing ones",
                            sub_folder_name, folder_path
                        ));
                    }

                    let sub_folder_name = format!("/{}", compound_path_parts[2..].join("/"));

                    sub_folders_config.insert(
                        fisrt_part,
                        normalize_folder_config(
                            &ParsedFolderConfig::Ok(CorrectParsedFolderConfig {
                                rules: None,
                                has_files: None,
                                folders: BTreeMap::from([(
                                    sub_folder_name,
                                    sub_folder_config.clone(),
                                )]),
                            }),
                            folder_path.clone(),
                        )?,
                    );
                } else {
                    let folder_path = format!("{}{}", folder_path, sub_folder_name);

                    sub_folders_config.insert(
                        sub_folder_name.clone(),
                        normalize_folder_config(sub_folder_config, folder_path)?,
                    );
                }
            }

            Ok(FolderConfig {
                file_rules,
                sub_folders_config,
                folder_rules,
            })
        }
    }
}

pub fn get_config(parsed_config: &ParsedConfig) -> Result<Config, String> {
    let (global_files_rules, global_folders_rules) = match &parsed_config.global_rules {
        Some(global_rules) => normalize_rules(global_rules, &String::from("global_config"))?,
        None => (vec![], vec![]),
    };

    Ok(Config {
        global_files_rules,
        global_folders_rules,
        root_folder: normalize_folder_config(&parsed_config.root_folder, String::from("."))?,
    })
}
