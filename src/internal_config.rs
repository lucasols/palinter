use std::collections::{BTreeMap, HashMap, HashSet};

use serde_yaml::Value;

use crate::parse_config_file::{
    CorrectParsedFolderConfig, ParsedAnyOr, ParsedBlocks, ParsedConfig, ParsedFileConditions,
    ParsedFileContentMatches, ParsedFileContentMatchesItem, ParsedFileExpect, ParsedFolderConfig,
    ParsedFolderExpect, ParsedRule, SingleOrMultiple,
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
pub struct FileConditions {
    pub has_extension: Option<Vec<String>>,
    pub has_name: Option<String>,
    pub not_has_name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Matches {
    Any(Vec<String>),
    All(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct ContentMatches {
    pub matches: Matches,
    pub at_least: usize,
    pub at_most: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct FileExpect {
    pub name_case_is: Option<NameCase>,
    pub extension_is: Option<Vec<String>>,
    pub has_sibling_file: Option<String>,
    pub content_matches: Option<Vec<ContentMatches>>,
    pub content_matches_some: Option<Vec<ContentMatches>>,
    pub name_is: Option<String>,
    pub name_is_not: Option<String>,

    pub error_msg: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RootFilesFindPattern {
    pub pattern: String,
    pub at_least: usize,
    pub at_most: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct FolderConditions {
    pub has_name_case: Option<NameCase>,
    pub has_name: Option<String>,
    pub not_has_name: Option<String>,
    pub root_files_find_pattern: Option<RootFilesFindPattern>,
}

#[derive(Debug, Clone)]
pub struct FolderExpect {
    pub name_case_is: Option<NameCase>,
    pub name_is: Option<String>,
    pub name_is_not: Option<String>,
    pub root_files_has: Option<String>,
    pub root_files_has_not: Option<String>,

    pub error_msg: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FileRule {
    pub conditions: AnyOr<FileConditions>,
    pub expect: AnyOr<Vec<FileExpect>>,
    pub non_recursive: bool,
    pub not_touch: bool,
    pub error_msg: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FolderRule {
    pub conditions: AnyOr<FolderConditions>,
    pub expect: AnyOr<Vec<FolderExpect>>,
    pub non_recursive: bool,
    pub not_touch: bool,
    pub error_msg: Option<String>,
}

#[derive(Debug, Default)]
pub struct OneOfFile {
    pub rules: Vec<FileRule>,
    pub error_msg: String,
}

#[derive(Debug, Default)]
pub struct OneOfFolder {
    pub rules: Vec<FolderRule>,
    pub error_msg: String,
}

#[derive(Debug, Default)]
pub struct OneOfBlocks {
    pub file_blocks: Vec<OneOfFile>,
    pub folder_blocks: Vec<OneOfFolder>,
}

#[derive(Debug)]
pub struct FolderConfig {
    pub sub_folders_config: HashMap<String, FolderConfig>,
    pub file_rules: Vec<FileRule>,
    pub folder_rules: Vec<FolderRule>,
    pub optional: bool,
    pub one_of_blocks: OneOfBlocks,
    pub allow_unconfigured_files: bool,
    pub allow_unconfigured_folders: bool,
}

#[derive(Debug)]
pub struct Config {
    pub root_folder: FolderConfig,
    pub analyze_content_of_files_types: Option<Vec<String>>,
    pub ignore: HashSet<String>,
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

fn get_true_flag(
    config_path: &String,
    flag: &Option<bool>,
    flag_name: &str,
) -> Result<bool, String> {
    match flag {
        Some(true) => Ok(true),
        Some(false) => Err(format!(
            "Config error in '{}': Invalid '{}' flag with false value, remove the flag if you don't want to use it",
            config_path, flag_name
        )),
        None => Ok(false),
    }
}

type NormalizedBlocks = BTreeMap<String, Vec<ParsedRule>>;

fn normalize_rules(
    rules: &Vec<ParsedRule>,
    config_path: &String,
    normalized_blocks: &NormalizedBlocks,
    config: &ParsedConfig,
) -> Result<(Vec<FileRule>, Vec<FolderRule>, OneOfBlocks), String> {
    let mut file_rules: Vec<FileRule> = vec![];
    let mut folder_rules: Vec<FolderRule> = vec![];
    let mut one_of_file_blocks: Vec<OneOfFile> = vec![];
    let mut one_of_folder_blocks: Vec<OneOfFolder> = vec![];

    for rule in rules {
        match rule {
            ParsedRule::File {
                conditions: parsed_conditions,
                expect,
                non_recursive,
                error_msg,
                expect_one_of,
                not_touch,
            } => {
                let conditions = match parsed_conditions {
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
                            has_name: conditions.has_name.clone(),
                            not_has_name: conditions.not_has_name.clone(),
                        })
                    }
                };

                check_rules_expects(expect, expect_one_of, config_path)?;

                if let Some(expect) = expect {
                    let new_expect: AnyOr<Vec<FileExpect>> = match &**expect {
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

                                expects.push(get_file_expect(
                                    parsed_expected,
                                    config_path,
                                    config,
                                )?);
                            }

                            AnyOr::Or(expects)
                        }
                    };

                    file_rules.push(FileRule {
                        conditions: conditions.clone(),
                        expect: new_expect,
                        error_msg: error_msg.clone(),
                        not_touch: get_true_flag(config_path, not_touch, "not_touch")?,
                        non_recursive: get_true_flag(config_path, non_recursive, "non_recursive")?,
                    });
                };

                if let Some(expect_one_of) = expect_one_of {
                    check_expect_one_of(config_path, expect_one_of.len(), &conditions)?;

                    if let Some(error_msg) = error_msg {
                        let mut rules: Vec<FileRule> = Vec::new();

                        for rule in expect_one_of {
                            rules.push(FileRule {
                                conditions: conditions.clone(),
                                expect: AnyOr::Or(vec![get_file_expect(
                                    rule.clone(),
                                    config_path,
                                    config,
                                )?]),
                                not_touch: get_true_flag(config_path, not_touch, "not_touch")?,
                                error_msg: None,
                                non_recursive: get_true_flag(
                                    config_path,
                                    non_recursive,
                                    "non_recursive",
                                )?,
                            });
                        }

                        let one_of = OneOfFile {
                            error_msg: error_msg.clone(),
                            rules,
                        };

                        one_of_file_blocks.push(one_of);
                    } else {
                        return Err(format!(
                            "Config error in '{}': rules with 'expect_one_of' property should have an error message, add one with the 'error_msg' property",
                            config_path
                        ));
                    }
                }
            }
            ParsedRule::Folder {
                conditions: parsed_conditions,
                error_msg,
                expect,
                non_recursive,
                expect_one_of,
                not_touch,
            } => {
                let conditions = match parsed_conditions {
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
                            has_name: conditions.has_name.clone(),
                            not_has_name: conditions.not_has_name.clone(),
                            root_files_find_pattern: conditions
                                .root_files_find_pattern
                                .as_ref()
                                .map(|root_files_find_pattern| RootFilesFindPattern {
                                    pattern: root_files_find_pattern.pattern.clone(),
                                    at_least: root_files_find_pattern.at_least.unwrap_or(1),
                                    at_most: root_files_find_pattern.at_most,
                                }),
                        })
                    }
                };

                check_rules_expects(expect, expect_one_of, config_path)?;

                if let Some(expect) = expect {
                    let new_expect = match &**expect {
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

                                expects.push(get_function_expect(parsed_expected, config_path)?);
                            }

                            AnyOr::Or(expects)
                        }
                    };

                    folder_rules.push(FolderRule {
                        conditions: conditions.clone(),
                        error_msg: error_msg.clone(),
                        expect: new_expect,
                        not_touch: get_true_flag(config_path, not_touch, "not_touch")?,
                        non_recursive: get_true_flag(config_path, non_recursive, "non_recursive")?,
                    });
                }

                if let Some(expect_one_of) = expect_one_of {
                    check_expect_one_of(config_path, expect_one_of.len(), &conditions)?;

                    if let Some(error_msg) = error_msg {
                        let mut rules: Vec<FolderRule> = Vec::new();

                        for rule_expect in expect_one_of {
                            rules.push(FolderRule {
                                conditions: conditions.clone(),
                                expect: AnyOr::Or(vec![get_function_expect(
                                    rule_expect.clone(),
                                    config_path,
                                )?]),
                                not_touch: get_true_flag(config_path, not_touch, "not_touch")?,
                                error_msg: None,
                                non_recursive: get_true_flag(
                                    config_path,
                                    non_recursive,
                                    "non_recursive",
                                )?,
                            });
                        }

                        let one_of = OneOfFolder {
                            error_msg: error_msg.clone(),
                            rules,
                        };

                        one_of_folder_blocks.push(one_of);
                    }
                }
            }
            ParsedRule::Block(block_id) => {
                let rules = normalized_blocks.get(block_id).ok_or(format!(
                    "Config error: Block '{}' in '{}' rules not found",
                    block_id, config_path
                ))?;

                let (block_file_rules, block_folder_rules, block_one_of_blocks) =
                    normalize_rules(rules, config_path, normalized_blocks, config)?;

                file_rules.extend(block_file_rules);
                folder_rules.extend(block_folder_rules);
                one_of_file_blocks.extend(block_one_of_blocks.file_blocks);
                one_of_folder_blocks.extend(block_one_of_blocks.folder_blocks);
            }
            ParsedRule::OneOf { rules, error_msg } => {
                if config_path.starts_with("global_rules") {
                    return Err(
                        "Config error: 'one_of' are not allowed in global rules".to_string()
                    );
                }

                let config_path = &format!("{}.{}", config_path, "one_of");

                if let Some(error_msg) = error_msg {
                    let mut one_of_file: Vec<FileRule> = vec![];
                    let mut one_of_folder: Vec<FolderRule> = vec![];

                    for rule in rules {
                        if let ParsedRule::OneOf { .. } = rule {
                            return Err(format!(
                                "Config error in '{}': Nested 'one_of' is not allowed",
                                config_path
                            ));
                        }

                        let (and_file_rules, and_folder_rules, _) = normalize_rules(
                            &vec![rule.clone()],
                            config_path,
                            normalized_blocks,
                            config,
                        )?;

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
                        one_of_file_blocks.push(OneOfFile {
                            rules: one_of_file,
                            error_msg: error_msg.clone(),
                        });
                    } else {
                        one_of_folder_blocks.push(OneOfFolder {
                            rules: one_of_folder,
                            error_msg: error_msg.clone(),
                        });
                    }
                } else {
                    return Err(
                        format!("Config error in '{}': 'one_of' must have an error message, add one with the 'error_msg' property",
                        config_path
                        )
                    );
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

fn check_expect_one_of<T>(
    config_path: &String,
    expect_one_of_len: usize,
    conditions: &AnyOr<T>,
) -> Result<(), String> {
    if config_path.starts_with("global_rules") {
        return Err(format!(
                "Config error in '{}': rules with 'expect_one_of' property are not allowed in global_rules",
                config_path
            ));
    }

    if expect_one_of_len < 2 {
        return Err(format!(
            "Config error in '{}': rules with 'expect_one_of' property should have at least 2 expect rules",
            config_path
        ));
    }

    if let AnyOr::Any = conditions {
        return Err(format!(
            "Config error in '{}': rules with 'expect_one_of' property cannot have 'any' condition",
            config_path
        ));
    }

    Ok(())
}

fn get_function_expect(
    parsed_expected: ParsedFolderExpect,
    config_path: &String,
) -> Result<FolderExpect, String> {
    Ok(FolderExpect {
        error_msg: parsed_expected.error_msg.clone(),
        name_is: parsed_expected.name_is,
        name_case_is: parsed_expected
            .name_case_is
            .as_ref()
            .map(|name_case| normalize_name_case(name_case, config_path))
            .transpose()?,
        name_is_not: parsed_expected.name_is_not,
        root_files_has: parsed_expected.root_files_has,
        root_files_has_not: parsed_expected.root_files_has_not,
    })
}

fn get_file_expect(
    parsed_expected: ParsedFileExpect,
    config_path: &String,
    parsed_config: &ParsedConfig,
) -> Result<FileExpect, String> {
    if (parsed_expected.content_matches.is_some() || parsed_expected.content_matches_any.is_some())
        && parsed_config.analyze_content_of_files_types.is_none()
    {
        return Err(format!(
            "Config error in '{}': to use 'content_matches' and 'content_matches_any' you must specify the 'analyze_content_of_files_types' property with the file extensions you want to analyze",
            config_path
        ));
    }

    Ok(FileExpect {
        error_msg: parsed_expected.error_msg,
        name_is: parsed_expected.name_is,
        extension_is: normalize_single_or_multiple_some(&parsed_expected.extension_is),
        name_case_is: parsed_expected
            .name_case_is
            .as_ref()
            .map(|name_case| normalize_name_case(name_case, config_path))
            .transpose()?,
        has_sibling_file: parsed_expected.has_sibling_file,
        content_matches: normalize_content_matches(parsed_expected.content_matches, config_path),
        content_matches_some: normalize_content_matches(
            parsed_expected.content_matches_any,
            config_path,
        ),
        name_is_not: parsed_expected.name_is_not,
    })
}

fn normalize_content_matches(
    parsed_content_matches: Option<ParsedFileContentMatches>,
    config_path: &String,
) -> Option<Vec<ContentMatches>> {
    if let Some(content_matches) = parsed_content_matches {
        match content_matches {
            ParsedFileContentMatches::Single(match_text) => Some(vec![ContentMatches {
                at_least: 1,
                at_most: None,
                matches: Matches::All(vec![match_text]),
            }]),
            ParsedFileContentMatches::Multiple(items) => Some(
                items
                    .iter()
                    .map(|item| -> ContentMatches {
                        match item {
                            ParsedFileContentMatchesItem::Single(match_text) => ContentMatches {
                                at_least: 1,
                                at_most: None,
                                matches: Matches::All(vec![match_text.clone()]),
                            },
                            ParsedFileContentMatchesItem::Config(config) => ContentMatches {
                                at_least: config.at_least.unwrap_or(1),
                                at_most: config.at_most,
                                matches: {
                                    if let Some(matches) = &config.all {
                                        Matches::All(matches.clone())
                                    } else if let Some(matches) = &config.any {
                                        Matches::Any(matches.clone())
                                    } else {
                                        Matches::All(vec![])
                                    }
                                },
                            },
                            ParsedFileContentMatchesItem::Error(error) => {
                                panic!(
                                    "Config error in {}: Invalid content_matches: {:#?}",
                                    config_path, error
                                )
                            }
                        }
                    })
                    .collect::<Vec<ContentMatches>>(),
            ),
            ParsedFileContentMatches::Error(error) => {
                panic!(
                    "Config error in {}: Invalid content_matches: {:#?}",
                    config_path, error
                )
            }
        }
    } else {
        None
    }
}

fn check_rules_expects<T, B>(
    expect: &Option<T>,
    expect_one_of: &Option<B>,
    config_path: &String,
) -> Result<(), String> {
    if expect.is_none() && expect_one_of.is_none() {
        return Err(format!(
            "Config error in '{}': missing 'expect' or 'expect_one_of'",
            config_path
        ));
    }

    if expect.is_some() && expect_one_of.is_some() {
        return Err(format!(
            "Config error in '{}': cannot have both 'expect' and 'expect_one_of'",
            config_path
        ));
    }

    Ok(())
}

pub fn normalize_folder_config(
    folder_config: &ParsedFolderConfig,
    folder_path: String,
    normalize_blocks: &NormalizedBlocks,
    parsed_config: &ParsedConfig,
) -> Result<FolderConfig, String> {
    match folder_config {
        ParsedFolderConfig::Error(wrong_value) => Err(format!(
            "Config error: Invalid folder config in '{}', received: {:#?}",
            folder_path, wrong_value
        )),
        ParsedFolderConfig::Ok(config) => {
            let mut rules: Vec<ParsedRule> = vec![];

            if let Some(files) = &config.has_files_in_root {
                let has_file_rules: Vec<ParsedRule> = files
                    .iter()
                    .map(|file| ParsedRule::File {
                        conditions: ParsedAnyOr::Conditions(ParsedFileConditions {
                            has_name: Some(file.clone()),
                            ..Default::default()
                        }),
                        expect: Some(Box::new(ParsedAnyOr::Any("any".to_string()))),
                        expect_one_of: None,
                        error_msg: None,
                        non_recursive: Some(true),
                        not_touch: None,
                    })
                    .collect();

                rules.extend(has_file_rules);
            }

            if let Some(parsed_rule) = &config.rules {
                rules.extend(parsed_rule.clone());
            }

            let (file_rules, folder_rules, one_of_blocks) =
                normalize_rules(&rules, &folder_path, normalize_blocks, parsed_config)?;

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
                                has_files_in_root: None,
                                optional: None,
                                folders: BTreeMap::from([(
                                    sub_folder_name,
                                    sub_folder_config.clone(),
                                )]),
                                allow_unconfigured_files: None,
                                allow_unconfigured_folders: None,
                            }),
                            folder_path.clone(),
                            normalize_blocks,
                            parsed_config,
                        )?,
                    );
                } else {
                    let folder_path = format!("{}{}", folder_path, sub_folder_name);

                    sub_folders_config.insert(
                        sub_folder_name.clone(),
                        normalize_folder_config(
                            sub_folder_config,
                            folder_path,
                            normalize_blocks,
                            parsed_config,
                        )?,
                    );
                }
            }

            let default_allow_unconfigured_files_or_folders = folder_path == ".";

            Ok(FolderConfig {
                file_rules,
                sub_folders_config,
                folder_rules,
                one_of_blocks,
                allow_unconfigured_files: config
                    .allow_unconfigured_files
                    .unwrap_or(default_allow_unconfigured_files_or_folders),
                allow_unconfigured_folders: config
                    .allow_unconfigured_folders
                    .unwrap_or(default_allow_unconfigured_files_or_folders),
                optional: get_true_flag(&folder_path, &config.optional, "optional")?,
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

    Ok(Config {
        root_folder: normalize_folder_config(
            &parsed_config.root_folder,
            String::from("."),
            normalized_block,
            parsed_config,
        )?,
        ignore: HashSet::from_iter(
            [
                parsed_config.ignore.clone().unwrap_or_default(),
                vec!["node_modules".to_string(), ".git".to_string()],
            ]
            .concat(),
        ),
        analyze_content_of_files_types: None,
    })
}
