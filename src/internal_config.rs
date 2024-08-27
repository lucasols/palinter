use std::collections::{BTreeMap, HashMap, HashSet};

use serde_yaml::Value;

use crate::{
    parse_config_file::{
        CorrectParsedFolderConfig, ParsedAnyNoneOrConditions, ParsedBlocks,
        ParsedConfig, ParsedFileConditions, ParsedFileContentMatches,
        ParsedFileContentMatchesItem, ParsedFileExpect, ParsedFolderConfig,
        ParsedFolderExpect, ParsedMatchImport, ParsedRule, SingleOrMultiple,
    },
    utils::clone_extend_vec,
};

#[derive(Debug, Clone)]
pub enum AnyOr<T> {
    Any,
    Or(T),
}

#[derive(Debug, Clone)]
pub enum AnyNoneOr<T> {
    Any,
    None,
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
pub enum MatchImport {
    From(String),
    DefaultFrom(String),
    Named { from: String, name: String },
}

#[derive(Debug, Clone)]
pub struct TsFileExpect {
    pub not_have_unused_exports: bool,
    pub not_have_circular_deps: bool,
    pub not_have_direct_circular_deps: bool,
    pub not_have_deps_from: Option<Vec<String>>,
    pub not_have_deps_outside: Option<Vec<String>>,
    pub not_have_exports_used_outside: Option<Vec<String>>,
    pub have_imports: Option<Vec<MatchImport>>,
    pub not_have_imports: Option<Vec<MatchImport>>,
}

#[derive(Debug, Clone)]
pub struct FileExpect {
    pub name_case_is: Option<NameCase>,
    pub extension_is: Option<Vec<String>>,
    pub have_sibling_file: Option<String>,
    pub content_matches: Option<Vec<ContentMatches>>,
    pub content_matches_some: Option<Vec<ContentMatches>>,
    pub content_not_matches: Option<Vec<String>>,
    pub name_is: Option<String>,
    pub name_is_not: Option<String>,
    pub ts: Option<TsFileExpect>,
    pub is_not_empty: bool,

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
    pub have_min_children: Option<usize>,
    pub child_rules: Option<(Vec<FolderRule>, Vec<FileRule>)>,

    pub error_msg: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FileRule {
    pub conditions: AnyOr<FileConditions>,
    pub expect: AnyNoneOr<Vec<FileExpect>>,
    pub non_recursive: bool,
    pub not_touch: bool,
    pub ignore_in_config_tests: bool,
    pub error_msg: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FolderRule {
    pub conditions: AnyOr<FolderConditions>,
    pub expect: AnyNoneOr<Vec<FolderExpect>>,
    pub non_recursive: bool,
    pub not_touch: bool,
    pub error_msg: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct OneOfFile {
    pub rules: Vec<FileRule>,
    pub error_msg: String,
}

#[derive(Debug, Default, Clone)]
pub struct OneOfFolder {
    pub rules: Vec<FolderRule>,
    pub error_msg: String,
}

#[derive(Debug, Default, Clone)]
pub struct OneOfBlocks {
    pub file_blocks: Vec<OneOfFile>,
    pub folder_blocks: Vec<OneOfFolder>,
}

#[derive(Debug, Clone)]
pub struct FolderConfig {
    pub sub_folders_config: HashMap<String, FolderConfig>,
    pub file_rules: Vec<FileRule>,
    pub folder_rules: Vec<FolderRule>,
    pub optional: bool,
    pub one_of_blocks: OneOfBlocks,
    pub allow_unexpected_files: bool,
    pub allow_unexpected_folders: bool,
    pub unexpected_files_error_msg: Option<String>,
    pub unexpected_folders_error_msg: Option<String>,
    pub unexpected_error_msg: Option<String>,
    pub append_error_msg: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TsConfig {
    pub aliases: HashMap<String, String>,
    pub unused_exports_entry_points: Vec<String>,
}

pub type ErrorMsgVars = Option<BTreeMap<String, String>>;

#[derive(Debug, Clone)]
pub struct Config {
    pub root_folder: FolderConfig,
    pub analyze_content_of_files_types: Vec<String>,
    pub ignore: HashSet<String>,
    pub ts_config: Option<TsConfig>,
    pub error_msg_vars: ErrorMsgVars,
}

fn normalize_single_or_multiple<T: Clone>(
    single_or_multiple: &SingleOrMultiple<T>,
) -> Vec<T> {
    match single_or_multiple {
        SingleOrMultiple::Single(single) => vec![single.clone()],
        SingleOrMultiple::Multiple(multiple) => multiple.to_vec(),
    }
}

pub fn normalize_single_or_multiple_option<T: Clone>(
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

fn check_any_or_none<T>(
    any: &String,
    config_path: &String,
) -> Result<AnyNoneOr<T>, String> {
    match any.as_str() {
        "any" => Ok(AnyNoneOr::Any),
        "none" => Ok(AnyNoneOr::None),
        _ => Err(format!(
            "Config error: Invalid any '{}' in '{}' rules, should be 'any' or 'none'",
            any, config_path
        )),
    }
}

fn normalize_name_case(
    name_case: &String,
    config_path: &String,
) -> Result<NameCase, String> {
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
                ignore_in_config_tests,
            } => {
                let conditions = match parsed_conditions {
                    ParsedAnyNoneOrConditions::AnyOrNone(any) => {
                        check_any(any, config_path)?;
                        AnyOr::Any
                    }
                    ParsedAnyNoneOrConditions::Conditions(conditions) => {
                        let ParsedFileConditions {
                            has_extension,
                            has_name,
                            not_has_name,
                            is_ts,
                            wrong,
                        } = conditions;

                        check_invalid_conditions(
                            wrong,
                            "if_file condition",
                            config_path,
                        )?;

                        let validated_is_ts =
                            get_true_flag(config_path, is_ts, "is_ts")?;

                        AnyOr::Or(FileConditions {
                            has_extension: (has_extension.is_some()
                                || validated_is_ts)
                                .then_some(clone_extend_vec(
                                    &normalize_single_or_multiple_option(
                                        has_extension,
                                    )
                                    .unwrap_or_default(),
                                    &validated_is_ts
                                        .then_some(vec![
                                            "ts".to_string(),
                                            "tsx".to_string(),
                                        ])
                                        .unwrap_or_default(),
                                )),
                            has_name: has_name.clone(),
                            not_has_name: not_has_name.clone(),
                        })
                    }
                };

                check_rules_expects(expect, expect_one_of, config_path)?;

                if let Some(expect) = expect {
                    let new_expect: AnyNoneOr<Vec<FileExpect>> = match &**expect {
                        ParsedAnyNoneOrConditions::AnyOrNone(any) => {
                            check_any_or_none(any, config_path)?
                        }
                        ParsedAnyNoneOrConditions::Conditions(expect_conditions) => {
                            let mut expects: Vec<FileExpect> = Vec::new();

                            for parsed_expected in
                                normalize_single_or_multiple(expect_conditions)
                            {
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

                            AnyNoneOr::Or(expects)
                        }
                    };

                    file_rules.push(FileRule {
                        conditions: conditions.clone(),
                        expect: new_expect,
                        error_msg: error_msg.clone(),
                        not_touch: get_true_flag(
                            config_path,
                            not_touch,
                            "not_touch",
                        )?,
                        non_recursive: get_true_flag(
                            config_path,
                            non_recursive,
                            "non_recursive",
                        )?,
                        ignore_in_config_tests: get_true_flag(
                            config_path,
                            ignore_in_config_tests,
                            "ignore_in_config_tests",
                        )?,
                    });
                };

                if let Some(expect_one_of) = expect_one_of {
                    check_expect_one_of(
                        config_path,
                        expect_one_of.len(),
                        &conditions,
                    )?;

                    if let Some(error_msg) = error_msg {
                        let mut rules: Vec<FileRule> = Vec::new();

                        for rule in expect_one_of {
                            rules.push(FileRule {
                                conditions: conditions.clone(),
                                expect: AnyNoneOr::Or(vec![get_file_expect(
                                    rule.clone(),
                                    config_path,
                                    config,
                                )?]),
                                not_touch: get_true_flag(
                                    config_path,
                                    not_touch,
                                    "not_touch",
                                )?,
                                error_msg: None,
                                non_recursive: get_true_flag(
                                    config_path,
                                    non_recursive,
                                    "non_recursive",
                                )?,
                                ignore_in_config_tests: get_true_flag(
                                    config_path,
                                    ignore_in_config_tests,
                                    "ignore_in_config_tests",
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
                    ParsedAnyNoneOrConditions::AnyOrNone(any) => {
                        check_any(any, config_path)?;
                        AnyOr::Any
                    }
                    ParsedAnyNoneOrConditions::Conditions(conditions) => {
                        check_invalid_conditions(
                            &conditions.wrong,
                            "if_folder condition",
                            config_path,
                        )?;

                        AnyOr::Or(FolderConditions {
                            has_name_case: conditions
                                .has_name_case
                                .as_ref()
                                .map(|name_case| {
                                    normalize_name_case(name_case, config_path)
                                })
                                .transpose()?,
                            has_name: conditions.has_name.clone(),
                            not_has_name: conditions.not_has_name.clone(),
                            root_files_find_pattern: conditions
                                .root_files_find_pattern
                                .as_ref()
                                .map(|root_files_find_pattern| {
                                    RootFilesFindPattern {
                                        pattern: root_files_find_pattern
                                            .pattern
                                            .clone(),
                                        at_least: root_files_find_pattern
                                            .at_least
                                            .unwrap_or(1),
                                        at_most: root_files_find_pattern.at_most,
                                    }
                                }),
                        })
                    }
                };

                check_rules_expects(expect, expect_one_of, config_path)?;

                if let Some(expect) = expect {
                    let new_expect = match &**expect {
                        ParsedAnyNoneOrConditions::AnyOrNone(any) => {
                            check_any_or_none(any, config_path)?
                        }
                        ParsedAnyNoneOrConditions::Conditions(expect_conditions) => {
                            let mut expects: Vec<FolderExpect> = Vec::new();

                            for parsed_expected in
                                normalize_single_or_multiple(expect_conditions)
                            {
                                check_invalid_conditions(
                                    &parsed_expected.wrong,
                                    "file expect condition",
                                    config_path,
                                )?;

                                expects.push(get_folder_expect(
                                    parsed_expected,
                                    config_path,
                                    config,
                                    normalized_blocks,
                                )?);
                            }

                            AnyNoneOr::Or(expects)
                        }
                    };

                    folder_rules.push(FolderRule {
                        conditions: conditions.clone(),
                        error_msg: error_msg.clone(),
                        expect: new_expect,
                        not_touch: get_true_flag(
                            config_path,
                            not_touch,
                            "not_touch",
                        )?,
                        non_recursive: get_true_flag(
                            config_path,
                            non_recursive,
                            "non_recursive",
                        )?,
                    });
                }

                if let Some(expect_one_of) = expect_one_of {
                    check_expect_one_of(
                        config_path,
                        expect_one_of.len(),
                        &conditions,
                    )?;

                    if let Some(error_msg) = error_msg {
                        let mut rules: Vec<FolderRule> = Vec::new();

                        for rule_expect in expect_one_of {
                            rules.push(FolderRule {
                                conditions: conditions.clone(),
                                expect: AnyNoneOr::Or(vec![get_folder_expect(
                                    rule_expect.clone(),
                                    config_path,
                                    config,
                                    normalized_blocks,
                                )?]),
                                not_touch: get_true_flag(
                                    config_path,
                                    not_touch,
                                    "not_touch",
                                )?,
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
            ParsedRule::Block(block_sring) => {
                let (block_id, custom_error, custom_not_touch, custom_non_recursive) = {
                    if block_sring.contains("::") {
                        let mut block_id = String::new();
                        let mut custom_error: Option<String> = None;
                        let mut custom_not_touch = None;
                        let mut custom_non_recursive = None;

                        for (i, part) in block_sring.split("::").enumerate() {
                            if i == 0 {
                                block_id = part.to_string();
                            } else if part.starts_with("error_msg=") {
                                custom_error = part
                                    .strip_prefix("error_msg=")
                                    .map(|s| s.to_string());
                            } else if part.starts_with("not_touch") {
                                custom_not_touch = if part == "not_touch=false" {
                                    Some(false)
                                } else {
                                    Some(true)
                                };
                            } else if part.starts_with("non_recursive") {
                                custom_non_recursive =
                                    if part == "non_recursive=false" {
                                        Some(false)
                                    } else {
                                        Some(true)
                                    };
                            }
                        }

                        (
                            block_id,
                            custom_error,
                            custom_not_touch,
                            custom_non_recursive,
                        )
                    } else {
                        (block_sring.clone(), None, None, None)
                    }
                };

                let rules = normalized_blocks
                    .get(&block_id)
                    .ok_or(format!(
                        "Config error: Block '{}' in '{}' rules not found",
                        block_id, config_path
                    ))?
                    .iter()
                    .map(|rule| match rule {
                        ParsedRule::File {
                            conditions,
                            expect,
                            expect_one_of,
                            non_recursive,
                            not_touch,
                            error_msg,
                            ignore_in_config_tests,
                        } => ParsedRule::File {
                            conditions: conditions.clone(),
                            expect: expect.clone(),
                            expect_one_of: expect_one_of.clone(),
                            non_recursive: custom_non_recursive.or(*non_recursive),
                            not_touch: custom_not_touch.or(*not_touch),
                            error_msg: custom_error.clone().or(error_msg.clone()),
                            ignore_in_config_tests: custom_non_recursive
                                .or(*ignore_in_config_tests),
                        },
                        ParsedRule::Folder {
                            conditions,
                            expect,
                            expect_one_of,
                            non_recursive,
                            not_touch,
                            error_msg,
                        } => ParsedRule::Folder {
                            conditions: conditions.clone(),
                            expect: expect.clone(),
                            expect_one_of: expect_one_of.clone(),
                            non_recursive: custom_non_recursive.or(*non_recursive),
                            not_touch: custom_not_touch.or(*not_touch),
                            error_msg: custom_error.clone().or(error_msg.clone()),
                        },
                        _ => rule.clone(),
                    })
                    .collect::<Vec<ParsedRule>>();

                let (block_file_rules, block_folder_rules, block_one_of_blocks) =
                    normalize_rules(&rules, config_path, normalized_blocks, config)?;

                file_rules.extend(block_file_rules);
                folder_rules.extend(block_folder_rules);
                one_of_file_blocks.extend(block_one_of_blocks.file_blocks);
                one_of_folder_blocks.extend(block_one_of_blocks.folder_blocks);
            }
            ParsedRule::OneOf { rules, error_msg } => {
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

                        if !and_file_rules.is_empty() && !and_folder_rules.is_empty()
                        {
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
                            || (!and_folder_rules.is_empty()
                                && !one_of_file.is_empty())
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

fn get_folder_expect(
    parsed_expected: ParsedFolderExpect,
    config_path: &String,
    parsed_config: &ParsedConfig,
    normalized_blocks: &NormalizedBlocks,
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
        have_min_children: parsed_expected.have_min_childs,
        child_rules: parsed_expected
            .child_rules
            .map(
                |rules| -> Result<(Vec<FolderRule>, Vec<FileRule>), String> {
                    let (file_rules, folder_rules, _) = normalize_rules(
                        &rules,
                        config_path,
                        normalized_blocks,
                        parsed_config,
                    )?;

                    for folder_rule in &folder_rules {
                        if let AnyNoneOr::Or(expect) = &folder_rule.expect {
                            if expect
                                .iter()
                                .any(|expect| expect.child_rules.is_some())
                            {
                                return Err(format!(
                                    "Config error in '{}': 'childs_rules' cannot be used inside another 'childs_rules'",
                                    config_path
                                ));
                            }
                        }
                    }

                    Ok((folder_rules, file_rules))
                },
            )
            .transpose()?,
    })
}

fn get_file_expect(
    parsed_expected: ParsedFileExpect,
    config_path: &String,
    parsed_config: &ParsedConfig,
) -> Result<FileExpect, String> {
    if (parsed_expected.content_matches.is_some()
        || parsed_expected.content_matches_any.is_some())
        && (parsed_config.analyze_content_of_files_types.is_none()
            && parsed_config.ts.is_none())
    {
        return Err(format!(
            "Config error in '{}': to use 'content_matches' and 'content_matches_any' you must specify the 'analyze_content_of_files_types' property with the file extensions you want to analyze",
            config_path
        ));
    }

    Ok(FileExpect {
        error_msg: parsed_expected.error_msg,
        name_is: parsed_expected.name_is,
        extension_is: normalize_single_or_multiple_option(
            &parsed_expected.extension_is,
        ),
        name_case_is: parsed_expected
            .name_case_is
            .as_ref()
            .map(|name_case| normalize_name_case(name_case, config_path))
            .transpose()?,
        have_sibling_file: parsed_expected.have_sibling_file,
        content_matches: normalize_content_matches(
            parsed_expected.content_matches,
            config_path,
        ),
        content_matches_some: normalize_content_matches(
            parsed_expected.content_matches_any,
            config_path,
        ),
        content_not_matches: normalize_single_or_multiple_option(
            &parsed_expected.content_not_matches,
        ),

        name_is_not: parsed_expected.name_is_not,
        is_not_empty: get_true_flag(
            config_path,
            &parsed_expected.is_not_empty,
            "is_not_empty",
        )?,
        ts: match parsed_expected.ts {
            Some(ts) => {
                if parsed_config.ts.is_none() {
                    return Err(format!(
                        "Config error in '{}': to use 'ts' assertions you must specify the 'ts' property",
                        config_path
                    ));
                }

                Some(TsFileExpect {
                    not_have_unused_exports: get_true_flag(
                        config_path,
                        &ts.not_have_unused_exports,
                        "ts.not_have_unused_exports",
                    )?,
                    not_have_circular_deps: get_true_flag(
                        config_path,
                        &ts.not_have_circular_deps,
                        "ts.not_have_circular_deps",
                    )?,
                    not_have_direct_circular_deps: get_true_flag(
                        config_path,
                        &ts.not_have_direct_circular_deps,
                        "ts.not_have_direct_circular_deps",
                    )?,
                    not_have_deps_from: normalize_single_or_multiple_option(
                        &ts.not_have_deps_from,
                    ),
                    not_have_deps_outside: normalize_single_or_multiple_option(
                        &ts.not_have_deps_outside,
                    ),
                    not_have_exports_used_outside:
                        normalize_single_or_multiple_option(
                            &ts.not_have_exports_used_outside,
                        ),
                    have_imports: ts.have_imports.map(|parsed_imports| {
                        normalize_parsed_match_import(parsed_imports)
                    }),
                    not_have_imports: ts.not_have_imports.map(|parsed_imports| {
                        normalize_parsed_match_import(parsed_imports)
                    }),
                })
            }
            None => None,
        },
    })
}

fn normalize_parsed_match_import(
    parsed_match_import: Vec<ParsedMatchImport>,
) -> Vec<MatchImport> {
    parsed_match_import
        .iter()
        .map(|parsed_match_import| -> MatchImport {
            if let Some(name) = &parsed_match_import.name {
                if name == "default" {
                    MatchImport::DefaultFrom(parsed_match_import.from.clone())
                } else {
                    MatchImport::Named {
                        name: name.clone(),
                        from: parsed_match_import.from.clone(),
                    }
                }
            } else {
                MatchImport::From(parsed_match_import.from.clone())
            }
        })
        .collect::<Vec<MatchImport>>()
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
                        conditions: ParsedAnyNoneOrConditions::Conditions(
                            ParsedFileConditions {
                                has_name: Some(file.clone()),
                                ..Default::default()
                            },
                        ),
                        expect: Some(Box::new(
                            ParsedAnyNoneOrConditions::AnyOrNone("any".to_string()),
                        )),
                        expect_one_of: None,
                        error_msg: None,
                        non_recursive: Some(true),
                        not_touch: None,
                        ignore_in_config_tests: None,
                    })
                    .collect();

                rules.extend(has_file_rules);
            }

            if let Some(parsed_rule) = &config.rules {
                rules.extend(parsed_rule.clone());
            }

            let (file_rules, folder_rules, one_of_blocks) = normalize_rules(
                &rules,
                &folder_path,
                normalize_blocks,
                parsed_config,
            )?;

            let mut sub_folders_config: HashMap<String, FolderConfig> =
                HashMap::new();

            for (sub_folder_name, sub_folder_config) in &config.folders {
                if !sub_folder_name.starts_with('/') {
                    return Err(format!(
                        "Config error: Invalid sub folder name: '{}' in '{}', folders name should start with '/'",
                        sub_folder_name, folder_path
                    ));
                }

                let compound_path_parts =
                    sub_folder_name.split('/').collect::<Vec<&str>>();

                if compound_path_parts.len() > 2 {
                    let fisrt_part = format!("/{}", compound_path_parts[1]);

                    if config.folders.contains_key(&fisrt_part) {
                        return Err(format!(
                            "Config error: Duplicate compound folder path: '{}' in '{}', compound folder paths should not conflict with existing ones",
                            sub_folder_name, folder_path
                        ));
                    }

                    let sub_folder_name =
                        format!("/{}", compound_path_parts[2..].join("/"));

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
                                allow_unexpected_files: None,
                                allow_unexpected_folders: None,
                                append_error_msg: None,
                                unexpected_error_msg: None,
                                allow_unexpected: None,
                                unexpected_files_error_msg: None,
                                unexpected_folders_error_msg: None,
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

            let default_allow_unexpected_files_or_folders =
                config.allow_unexpected.unwrap_or(false) || folder_path == ".";

            Ok(FolderConfig {
                file_rules,
                sub_folders_config,
                folder_rules,
                one_of_blocks,
                append_error_msg: config.append_error_msg.clone(),
                unexpected_files_error_msg: config
                    .unexpected_files_error_msg
                    .clone(),
                unexpected_folders_error_msg: config
                    .unexpected_folders_error_msg
                    .clone(),
                unexpected_error_msg: config.unexpected_error_msg.clone(),
                allow_unexpected_files: config
                    .allow_unexpected_files
                    .unwrap_or(default_allow_unexpected_files_or_folders),
                allow_unexpected_folders: config
                    .allow_unexpected_folders
                    .unwrap_or(default_allow_unexpected_files_or_folders),
                optional: get_true_flag(&folder_path, &config.optional, "optional")?,
            })
        }
    }
}

fn normalize_blocks(
    parsed_blocks: &ParsedBlocks,
) -> Result<NormalizedBlocks, String> {
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
    if !parsed_config.wrong.is_empty() {
        return Err(format!(
            "Config error: Invalid config, received: {:#?}",
            parsed_config.wrong
        ));
    }

    let normalized_block = &normalize_blocks(&parsed_config.blocks)?;

    let mut analyze_content_of_files_types = parsed_config
        .analyze_content_of_files_types
        .clone()
        .map(|types| {
            types
                .iter()
                .map(|type_| type_.to_string())
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();

    if parsed_config.ts.is_some() {
        analyze_content_of_files_types
            .extend(vec!["ts".to_string(), "tsx".to_string()]);
    }

    Ok(Config {
        error_msg_vars: parsed_config.error_msg_vars.clone(),
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
        analyze_content_of_files_types,
        ts_config: parsed_config.ts.as_ref().map(|ts| TsConfig {
            aliases: HashMap::from_iter(ts.aliases.clone()),
            unused_exports_entry_points: ts.unused_exports_entry_points.clone(),
        }),
    })
}
