use std::collections::{BTreeMap, HashMap};

use serde_yaml::Value;

use crate::parse_config_file::{
    CorrectParsedFolderConfig, ParsedAnyOr, ParsedBlocks, ParsedConfig, ParsedFolderConfig,
    ParsedRule, SingleOrMultiple,
};

#[derive(Debug, Clone)]
pub enum AnyOr<T> {
    Any,
    Or(T),
}

#[derive(Debug, Clone)]
pub enum NameCase {
    Camel,
    Snake,
    Kebab,
    Pascal,
    Constant,
}

#[derive(Debug, Clone)]
pub struct FileExpect {
    pub name_case_is: Option<NameCase>,
    pub extension_is: Option<Vec<String>>,

    pub error_msg: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FolderExpect {
    pub name_case_is: Option<NameCase>,
    pub error_msg: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FileConditions {
    pub has_extension: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct FolderConditions {
    pub has_name_case: Option<NameCase>,
}

#[derive(Debug, Clone)]
pub struct FileRule {
    pub conditions: AnyOr<FileConditions>,
    pub expect: AnyOr<Vec<FileExpect>>,
    pub non_recursive: bool,
    pub error_msg: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FolderRule {
    pub conditions: AnyOr<FolderConditions>,
    pub expect: AnyOr<Vec<FolderExpect>>,
    pub non_recursive: bool,
    pub error_msg: Option<String>,
}

pub struct Folder {
    pub name: String,
    pub rules: Vec<FileRule>,
}

#[derive(Debug, Default)]
pub struct OneOfBlocks {
    pub file_blocks: Vec<Vec<FileRule>>,
    pub folder_blocks: Vec<Vec<FolderRule>>,
}

#[derive(Debug)]
pub struct FolderConfig {
    pub sub_folders_config: HashMap<String, FolderConfig>,
    pub file_rules: Vec<FileRule>,
    pub folder_rules: Vec<FolderRule>,
    pub one_of_blocks: OneOfBlocks,
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

fn check_any(any: &String, config_path: &String) -> Result<(), String> {
    if any != "any" {
        Err(format!(
            "Config error: Invalid any '{}' in '{}' rules, should be 'any'",
            any, config_path
        ))
    } else {
        Ok(())
    }
}

fn normalize_name_case(name_case: &String, config_path: &String) -> Result<NameCase, String> {
    match name_case.as_str() {
        "camelCase" => Ok(NameCase::Camel),
        "snake_case" => Ok(NameCase::Snake),
        "kebab-case" => Ok(NameCase::Kebab),
        "PascalCase" => Ok(NameCase::Pascal),
        "CONSTANT_CASE" => Ok(NameCase::Constant),
        _ => Err(format!(
            "Config error: Invalid name_case_is '{}' in '{}' rules",
            name_case, config_path
        )),
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

type NormalizedBlocks = BTreeMap<String, Vec<ParsedRule>>;

fn normalize_rules(
    rules: &Vec<ParsedRule>,
    config_path: &String,
    normalized_blocks: &NormalizedBlocks,
) -> Result<(Vec<FileRule>, Vec<FolderRule>, OneOfBlocks), String> {
    let mut file_rules: Vec<FileRule> = vec![];
    let mut folder_rules: Vec<FolderRule> = vec![];
    let mut one_of_file_blocks: Vec<Vec<FileRule>> = vec![];
    let mut one_of_folder_blocks: Vec<Vec<FolderRule>> = vec![];

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
                        check_any(any, config_path)?;
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
                        check_any(any, config_path)?;
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
                                error_msg: parsed_expected.error_msg.clone(),
                                extension_is: normalize_single_or_multiple_some(
                                    &parsed_expected.extension_is,
                                ),
                                name_case_is: parsed_expected
                                    .name_case_is
                                    .as_ref()
                                    .map(|name_case| normalize_name_case(name_case, config_path))
                                    .transpose()?,
                            });
                        }

                        AnyOr::Or(expects)
                    }
                };

                file_rules.push(FileRule {
                    conditions,
                    expect: new_expect,
                    error_msg: error_msg.clone(),
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
                        check_any(any, config_path)?;
                        AnyOr::Any
                    }
                    ParsedAnyOr::Conditions(conditions) => {
                        check_invalid_conditions(
                            &conditions.wrong,
                            "if_folder condition",
                            config_path,
                        )?;

                        AnyOr::Or(FolderConditions {
                            has_name_case: conditions
                                .has_name_case
                                .as_ref()
                                .map(|name_case| normalize_name_case(name_case, config_path))
                                .transpose()?,
                        })
                    }
                };

                let new_expect = match expect {
                    ParsedAnyOr::Any(any) => {
                        check_any(any, config_path)?;
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
                                error_msg: parsed_expected.error_msg.clone(),
                                name_case_is: parsed_expected
                                    .name_case_is
                                    .as_ref()
                                    .map(|name_case| normalize_name_case(name_case, config_path))
                                    .transpose()?,
                            });
                        }

                        AnyOr::Or(expects)
                    }
                };

                folder_rules.push(FolderRule {
                    conditions,
                    error_msg: error_msg.clone(),
                    expect: new_expect,
                    non_recursive: non_recursive.unwrap_or(false),
                });
            }
            ParsedRule::Block(block_id) => {
                let rules = normalized_blocks.get(block_id).ok_or(format!(
                    "Config error: Block '{}' in '{}' rules not found",
                    block_id, config_path
                ))?;

                let (block_file_rules, block_folder_rules, block_one_of_blocks) =
                    normalize_rules(rules, config_path, normalized_blocks)?;

                file_rules.extend(block_file_rules);
                folder_rules.extend(block_folder_rules);
                one_of_file_blocks.extend(block_one_of_blocks.file_blocks);
                one_of_folder_blocks.extend(block_one_of_blocks.folder_blocks);
            }
            ParsedRule::OneOf { rules } => {
                if config_path.starts_with("global_rules") {
                    return Err(
                        "Config error: 'one_of' blocks are not allowed in global rules".to_string(),
                    );
                }

                let mut one_of_file: Vec<FileRule> = vec![];
                let mut one_of_folder: Vec<FolderRule> = vec![];

                let config_path = &format!("{}.{}", config_path, "one_of");

                for rule in rules {
                    if let ParsedRule::OneOf { .. } = rule {
                        return Err(format!(
                            "Config error in '{}': Nested 'one_of' is not allowed",
                            config_path
                        ));
                    }

                    let (and_file_rules, and_folder_rules, _) =
                        normalize_rules(&vec![rule.clone()], config_path, normalized_blocks)?;

                    if !and_file_rules.is_empty() && !and_folder_rules.is_empty() {
                        return Err(format!(
                            "Config error in '{}': Blocks used in 'one_of' cannot contain both file and folder rules",
                            config_path
                        ));
                    }

                    if and_file_rules.len() > 1 || and_folder_rules.len() > 1 {
                        return Err(format!(
                            "Config error in '{}': Blocks used in 'one_of' must not have more than one rule",
                            config_path
                        ));
                    }

                    for and_file_rule in &and_file_rules {
                        if let AnyOr::Any = and_file_rule.conditions {
                            return Err(format!(
                                "Config error in '{}': 'one_of' cannot contain rules with 'any' condition",
                                config_path
                            ));
                        }
                    }

                    for and_folder_rule in &and_folder_rules {
                        if let AnyOr::Any = and_folder_rule.conditions {
                            return Err(format!(
                                "Config error in '{}': 'one_of' cannot contain rules with 'any' condition",
                                config_path
                            ));
                        }
                    }

                    if (!and_file_rules.is_empty() && !one_of_folder.is_empty())
                        || (!and_folder_rules.is_empty() && !one_of_file.is_empty())
                    {
                        return Err(format!(
                            "Config error in '{}': 'one_of' block cannot contain both file and folder rules",
                            config_path
                        ));
                    }

                    one_of_file.extend(and_file_rules);
                    one_of_folder.extend(and_folder_rules);
                }

                if (!one_of_file.is_empty() && one_of_file.len() < 2)
                    || (!one_of_folder.is_empty() && one_of_folder.len() < 2)
                {
                    return Err(format!(
                        "Config error in '{}': 'one_of' must contain at least 2 rules",
                        config_path
                    ));
                }

                if !one_of_file.is_empty() {
                    one_of_file_blocks.push(one_of_file);
                } else {
                    one_of_folder_blocks.push(one_of_folder);
                }
            }
            ParsedRule::Error(error) => {
                return Err(format!(
                    "Config error: Invalid rule in '{}', received: {:#?}",
                    config_path, error
                ));
            }
        }
    }

    Ok((
        file_rules,
        folder_rules,
        OneOfBlocks {
            file_blocks: one_of_file_blocks,
            folder_blocks: one_of_folder_blocks,
        },
    ))
}

fn normalize_folder_config(
    folder_config: &ParsedFolderConfig,
    folder_path: String,
    normalize_blocks: &NormalizedBlocks,
) -> Result<FolderConfig, String> {
    match folder_config {
        ParsedFolderConfig::Error(wrong_value) => Err(format!(
            "Config error: Invalid folder config in '{}', received: {:#?}",
            folder_path, wrong_value
        )),
        ParsedFolderConfig::Ok(config) => {
            let (file_rules, folder_rules, one_of_blocks) = match &config.rules {
                Some(files) => normalize_rules(files, &folder_path, normalize_blocks)?,
                None => (vec![], vec![], OneOfBlocks::default()),
            };

            let mut sub_folders_config: HashMap<String, FolderConfig> = HashMap::new();

            for (sub_folder_name, sub_folder_config) in &config.folders {
                if !sub_folder_name.starts_with('/') {
                    return Err(format!(
                        "Config error: Invalid sub folder name: '{}' in '{}', folders name should start with '/'",
                        sub_folder_name, folder_path
                    ));
                }

                let compound_path_parts = sub_folder_name.split('/').collect::<Vec<&str>>();

                if compound_path_parts.len() > 2 {
                    let fisrt_part = format!("/{}", compound_path_parts[1]);

                    if config.folders.contains_key(&fisrt_part) {
                        return Err(format!(
                            "Config error: Duplicate compound folder path: '{}' in '{}', compound folder paths should not conflict with existing ones",
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
                            normalize_blocks,
                        )?,
                    );
                } else {
                    let folder_path = format!("{}{}", folder_path, sub_folder_name);

                    sub_folders_config.insert(
                        sub_folder_name.clone(),
                        normalize_folder_config(sub_folder_config, folder_path, normalize_blocks)?,
                    );
                }
            }

            Ok(FolderConfig {
                file_rules,
                sub_folders_config,
                folder_rules,
                one_of_blocks,
            })
        }
    }
}

fn normalize_blocks(parsed_blocks: &ParsedBlocks) -> Result<NormalizedBlocks, String> {
    let mut normalized_blocks: NormalizedBlocks = BTreeMap::new();

    if let Some(blocks) = parsed_blocks {
        for (block_name, block) in blocks {
            let rules = normalize_single_or_multiple(block);

            for rule in &rules {
                if let ParsedRule::Block(block_id) = rule {
                    return Err(format!(
                        "Config error: Block '{}' cannot be used inside another block",
                        block_id
                    ));
                }
            }

            normalized_blocks.insert(block_name.clone(), rules);
        }
    }

    Ok(normalized_blocks)
}

pub fn get_config(parsed_config: &ParsedConfig) -> Result<Config, String> {
    let normalized_block = &normalize_blocks(&parsed_config.blocks)?;

    let (global_files_rules, global_folders_rules, one_of_blocks) =
        match &parsed_config.global_rules {
            Some(global_rules) => normalize_rules(
                global_rules,
                &String::from("global_rules"),
                normalized_block,
            )?,
            None => (vec![], vec![], OneOfBlocks::default()),
        };

    Ok(Config {
        global_files_rules,
        global_folders_rules,
        root_folder: normalize_folder_config(
            &parsed_config.root_folder,
            String::from("."),
            normalized_block,
        )?,
    })
}
