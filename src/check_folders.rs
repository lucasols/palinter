use colored::Colorize;
use std::collections::HashSet;

use crate::{
    analyze_ts_deps::{
        ts_checks::{
            check_ts_not_have_circular_deps, check_ts_not_have_unused_exports,
        },
        UsedFilesDepsInfo,
    },
    internal_config::{
        AnyNoneOr, AnyOr, Config, FileConditions, FileExpect, FileRule,
        FolderConditions, FolderConfig, FolderExpect, FolderRule,
    },
    load_folder_structure::{File, Folder, FolderChild},
};

use self::checks::{
    check_content, check_content_not_matches, check_negated_path_pattern,
    check_negated_root_files_has_pattern, check_path_pattern,
    check_root_files_find_pattern, check_root_files_has_pattern, extension_is,
    has_sibling_file, name_case_is, path_pattern_match, Capture,
};

#[derive(Debug, Default)]
pub struct ConditionsResult {
    pub captures: Vec<Capture>,
}

fn file_matches_condition(
    file: &File,
    conditions: &AnyOr<FileConditions>,
) -> Option<ConditionsResult> {
    match conditions {
        AnyOr::Any => Some(ConditionsResult::default()),
        AnyOr::Or(conditions) => {
            let mut has_name_captures: Vec<Capture> = Vec::new();

            if let Some(extensions) = &conditions.has_extension {
                if !extensions.contains(&file.extension.clone().unwrap_or_default())
                {
                    return None;
                }
            }

            if let Some(pattern) = &conditions.has_name {
                if let Ok(captures) =
                    path_pattern_match(&file.name_with_ext, pattern)
                {
                    has_name_captures.extend(captures)
                } else {
                    return None;
                }
            }

            if let Some(pattern) = &conditions.not_has_name {
                if path_pattern_match(&file.name_with_ext, pattern).is_ok() {
                    return None;
                }
            }

            Some(ConditionsResult {
                captures: has_name_captures,
            })
        }
    }
}

fn append_expect_error(
    result: Result<(), String>,
    expect_error_msg: &Option<String>,
) -> Result<(), String> {
    match result {
        Ok(_) => Ok(()),
        Err(error) => {
            if let Some(expect_error_msg) = expect_error_msg {
                Err(format!(
                    "{}{}",
                    expect_error_msg,
                    format!(" | {}", error).dimmed()
                ))
            } else {
                Err(error)
            }
        }
    }
}

fn file_pass_expected(
    file: &File,
    expected: &AnyNoneOr<Vec<FileExpect>>,
    folder: &Folder,
    conditions_result: &ConditionsResult,
    used_files_deps_info: &mut UsedFilesDepsInfo,
) -> Result<(), Vec<String>> {
    if let AnyNoneOr::None = expected {
        return Err(vec!["File is not expected".to_string()]);
    }

    let mut errors = Vec::new();

    let mut check_result =
        |result: Result<(), String>, expect_error_msg: &Option<String>| {
            if let Err(error) = append_expect_error(result, expect_error_msg) {
                errors.push(error);
            }
        };

    if let AnyNoneOr::Or(expected) = expected {
        for expect in expected {
            let mut pass_some_expect = false;

            if let Some(file_name_case_is) = &expect.name_case_is {
                pass_some_expect = true;
                check_result(
                    name_case_is(&file.basename, file_name_case_is),
                    &expect.error_msg,
                );
            }

            if let Some(file_extension_is) = &expect.extension_is {
                pass_some_expect = true;
                check_result(
                    extension_is(&file.extension, file_extension_is),
                    &expect.error_msg,
                );
            }

            if let Some(sibling_file_pattern) = &expect.have_sibling_file {
                pass_some_expect = true;
                check_result(
                    has_sibling_file(
                        sibling_file_pattern,
                        folder,
                        &conditions_result.captures,
                    ),
                    &expect.error_msg,
                );
            }

            if let Some(content_matches) = &expect.content_matches {
                pass_some_expect = true;
                check_result(
                    check_content(
                        &file.content,
                        content_matches,
                        &conditions_result.captures,
                        false,
                    ),
                    &expect.error_msg,
                );
            }

            if let Some(content_matches_some) = &expect.content_matches_some {
                pass_some_expect = true;
                check_result(
                    check_content(
                        &file.content,
                        content_matches_some,
                        &conditions_result.captures,
                        true,
                    ),
                    &expect.error_msg,
                );
            }

            if let Some(name_is) = &expect.name_is {
                pass_some_expect = true;
                check_result(
                    check_path_pattern(
                        &file.name_with_ext,
                        name_is,
                        &conditions_result.captures,
                    ),
                    &expect.error_msg,
                );
            }

            if let Some(name_is_not) = &expect.name_is_not {
                pass_some_expect = true;
                check_result(
                    check_negated_path_pattern(
                        &file.name_with_ext,
                        name_is_not,
                        &conditions_result.captures,
                    ),
                    &expect.error_msg,
                );
            }

            if let Some(content_not_matches) = &expect.content_not_matches {
                pass_some_expect = true;
                check_result(
                    check_content_not_matches(
                        &file.content,
                        content_not_matches,
                        &conditions_result.captures,
                    ),
                    &expect.error_msg,
                );
            }

            if let Some(ts_expect) = &expect.ts {
                if ts_expect.not_have_unused_exports {
                    pass_some_expect = true;
                    check_result(
                        check_ts_not_have_unused_exports(file, used_files_deps_info),
                        &expect.error_msg,
                    );
                }

                if ts_expect.not_have_circular_deps {
                    pass_some_expect = true;
                    check_result(
                        check_ts_not_have_circular_deps(file, used_files_deps_info),
                        &expect.error_msg,
                    );
                }
            }

            if cfg!(debug_assertions) && !pass_some_expect {
                panic!("Unexpect expect {:#?}", expect);
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn folder_matches_condition(
    folder: &Folder,
    conditions: &AnyOr<FolderConditions>,
) -> Option<ConditionsResult> {
    match conditions {
        AnyOr::Any => Some(ConditionsResult::default()),
        AnyOr::Or(conditions) => {
            let mut result_captures: Vec<Capture> = Vec::new();

            if let Some(pattern) = &conditions.has_name_case {
                if name_case_is(&folder.name, pattern).is_err() {
                    return None;
                }
            }

            if let Some(pattern) = &conditions.has_name {
                if let Ok(captures) = path_pattern_match(&folder.name, pattern) {
                    result_captures.extend(captures);
                } else {
                    return None;
                }
            }

            if let Some(find_pattern) = &conditions.root_files_find_pattern {
                if let Ok(captures) =
                    check_root_files_find_pattern(folder, find_pattern)
                {
                    result_captures.extend(captures);
                } else {
                    return None;
                }
            }

            if let Some(pattern) = &conditions.not_has_name {
                if path_pattern_match(&folder.name, pattern).is_ok() {
                    return None;
                }
            }

            Some(ConditionsResult {
                captures: result_captures,
            })
        }
    }
}

fn folder_pass_expected(
    folder: &Folder,
    expected: &AnyNoneOr<Vec<FolderExpect>>,
    conditions_result: &ConditionsResult,
) -> Result<(), String> {
    match expected {
        AnyNoneOr::None => Err("Folder is not expected".to_string()),
        AnyNoneOr::Any => Ok(()),
        AnyNoneOr::Or(expected) => {
            let mut pass_some_expect = false;

            for expect in expected {
                if let Some(file_name_case_is) = &expect.name_case_is {
                    pass_some_expect = true;
                    append_expect_error(
                        name_case_is(&folder.name, file_name_case_is),
                        &expect.error_msg,
                    )?;
                }

                if let Some(name_is) = &expect.name_is {
                    pass_some_expect = true;
                    append_expect_error(
                        check_path_pattern(
                            &folder.name,
                            name_is,
                            &conditions_result.captures,
                        ),
                        &expect.error_msg,
                    )?;
                }

                if let Some(name_is_not) = &expect.name_is_not {
                    pass_some_expect = true;
                    append_expect_error(
                        check_negated_path_pattern(
                            &folder.name,
                            name_is_not,
                            &conditions_result.captures,
                        ),
                        &expect.error_msg,
                    )?;
                }

                if let Some(root_files_has) = &expect.root_files_has {
                    pass_some_expect = true;
                    append_expect_error(
                        check_root_files_has_pattern(
                            folder,
                            root_files_has,
                            &conditions_result.captures,
                        )
                        .map(|_| ()),
                        &expect.error_msg,
                    )?;
                }

                if let Some(root_files_has_not) = &expect.root_files_has_not {
                    pass_some_expect = true;
                    append_expect_error(
                        check_negated_root_files_has_pattern(
                            folder,
                            root_files_has_not,
                            &conditions_result.captures,
                        ),
                        &expect.error_msg,
                    )?;
                }

                if cfg!(debug_assertions) && !pass_some_expect {
                    panic!("Unexpect expect {:#?}", expect);
                }
            }

            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
struct InheritedFileRule {
    rule: FileRule,
}

#[derive(Debug, Clone)]
struct InheritedFolderRule {
    rule: FolderRule,
}

pub fn normalize_folder_config_name(name: &String) -> String {
    if name == "." {
        name.to_owned()
    } else {
        let normalized_name = name.strip_prefix('/');

        let name = if let Some(name) = normalized_name {
            name
        } else {
            panic!("Invalid folder config name: {}", name)
        };

        name.to_string()
    }
}

pub fn to_folder_config_name(name: &String) -> String {
    if name == "." {
        name.to_owned()
    } else {
        format!("/{}", name)
    }
}

fn check_folder_childs(
    folder: &Folder,
    folder_config: Option<&FolderConfig>,
    folder_path: String,
    inherited_files_rules: Vec<InheritedFileRule>,
    inherited_folders_rules: Vec<InheritedFolderRule>,
    used_files_deps_info: &mut UsedFilesDepsInfo,
) -> Result<(), Vec<String>> {
    let mut errors: Vec<String> = Vec::new();

    let allow_unconfigured_folders = folder_config.map_or(false, |folder_config| {
        folder_config.allow_unexpected_folders
    });
    let allow_unconfigured_files = folder_config
        .map_or(false, |folder_config| folder_config.allow_unexpected_files);

    let mut folders_missing_check = folder_config
        .map(|folder_config| {
            folder_config
                .sub_folders_config
                .iter()
                .filter_map(
                    |(name, config)| {
                        if config.optional {
                            None
                        } else {
                            Some(name)
                        }
                    },
                )
                .map(normalize_folder_config_name)
                .collect::<HashSet<String>>()
        })
        .unwrap_or_default();

    for child in &folder.childs {
        match child {
            FolderChild::FileChild(file) => {
                let mut file_touched = false;

                let file_error_prefix = format!(
                    "File {}\n • ",
                    format!("{}/{}:", folder_path, file.name_with_ext)
                        .bright_yellow()
                );

                let mut check_file_rule = |rule: &FileRule| {
                    if let Some(conditions_result) =
                        file_matches_condition(file, &rule.conditions)
                    {
                        if !rule.not_touch {
                            file_touched = true;
                        }

                        if let Err(expect_errors) = file_pass_expected(
                            file,
                            &rule.expect,
                            folder,
                            &conditions_result,
                            used_files_deps_info,
                        ) {
                            for error in expect_errors {
                                errors.push(format!(
                                    "{}{}",
                                    file_error_prefix,
                                    if let Some(custom_error) = &rule.error_msg {
                                        format!("{} | {}", custom_error, error)
                                    } else {
                                        error
                                    }
                                ));
                            }
                        }
                    }
                };

                if let Some(folder_config) = folder_config {
                    for rule in &folder_config.file_rules {
                        check_file_rule(rule)
                    }
                }

                for inherited_rule in &inherited_files_rules {
                    check_file_rule(&inherited_rule.rule)
                }

                if let Some(folder_config) = folder_config {
                    for one_of in &folder_config.one_of_blocks.file_blocks {
                        let mut one_of_matched_at_least_one_condition = false;
                        let mut one_of_matched = false;

                        for rule in &one_of.rules {
                            if let Some(conditions_result) =
                                file_matches_condition(file, &rule.conditions)
                            {
                                one_of_matched_at_least_one_condition = true;

                                if !rule.not_touch {
                                    file_touched = true;
                                }

                                if file_pass_expected(
                                    file,
                                    &rule.expect,
                                    folder,
                                    &conditions_result,
                                    used_files_deps_info,
                                )
                                .is_ok()
                                {
                                    one_of_matched = true;
                                    break;
                                }
                            }
                        }

                        if one_of_matched_at_least_one_condition && !one_of_matched {
                            errors.push(format!(
                                "{}{}",
                                file_error_prefix, one_of.error_msg
                            ));
                        }
                    }
                }

                if !file_touched && !allow_unconfigured_files {
                    errors.push(format!(
                        "File {} is not expected in folder {}{}",
                        file.name_with_ext.bright_yellow(),
                        folder_path.bright_red(),
                        folder_config
                            .and_then(|cfg| cfg.unexpected_files_error_msg.as_ref())
                            .map(|msg| format!(" | {}", msg))
                            .unwrap_or_default()
                    ));
                }
            }
            FolderChild::Folder(sub_folder) => {
                let folder_error_prefix = format!(
                    "Folder {}\n • ",
                    format!("{}/{}:", folder_path, sub_folder.name).bright_red()
                );

                let mut folder_touched = false;
                let mut folder_has_error = false;

                let mut check_folder_rule = |rule: &FolderRule| {
                    let folder_matches =
                        folder_matches_condition(sub_folder, &rule.conditions);

                    if let Some(conditions_result) = folder_matches {
                        folders_missing_check.remove(&sub_folder.name);

                        if !rule.not_touch {
                            folder_touched = true;
                        }

                        if let Err(error) = folder_pass_expected(
                            sub_folder,
                            &rule.expect,
                            &conditions_result,
                        ) {
                            folder_has_error = true;
                            errors.push(format!(
                                "{}{}",
                                folder_error_prefix,
                                if let Some(custom_error) = &rule.error_msg {
                                    format!("{} | {}", custom_error, error)
                                } else {
                                    error
                                }
                            ));
                        }
                    }
                };

                if let Some(folder_config) = folder_config {
                    for rule in &folder_config.folder_rules {
                        check_folder_rule(rule);
                    }
                }

                for inheridte_rule in &inherited_folders_rules {
                    check_folder_rule(&inheridte_rule.rule);
                }

                if folder_has_error {
                    continue;
                }

                let sub_folder_cfg: Option<&FolderConfig> = match folder_config {
                    Some(folder_config) => folder_config
                        .sub_folders_config
                        .get(&to_folder_config_name(&sub_folder.name)),
                    None => None,
                };

                let parent_path = if folder_path.is_empty() {
                    sub_folder.name.clone()
                } else {
                    format!("{}/{}", folder_path, sub_folder.name)
                };

                if sub_folder_cfg.is_some() {
                    folder_touched = true;
                    folders_missing_check.remove(&sub_folder.name);
                } else if !folder_touched && !allow_unconfigured_folders {
                    errors.push(format!(
                        "Folder {} is not expected in folder {}{}",
                        format!("/{}", sub_folder.name).bright_red(),
                        folder_path.bright_red(),
                        folder_config
                            .and_then(|cfg| cfg
                                .unexpected_folders_error_msg
                                .as_ref())
                            .map(|msg| format!(" | {}", msg))
                            .unwrap_or_default()
                    ));
                }

                if !folder_touched {
                    continue;
                }

                let parent_file_rules: Vec<InheritedFileRule> = folder_config
                    .map_or(Vec::new(), |folder_config| {
                        folder_config
                            .file_rules
                            .iter()
                            .filter_map(|rule| match rule.non_recursive {
                                true => None,
                                false => {
                                    Some(InheritedFileRule { rule: rule.clone() })
                                }
                            })
                            .collect()
                    });

                let sub_folder_inherited_files_rules =
                    [inherited_files_rules.clone(), parent_file_rules].concat();

                let parent_folder_rules: Vec<InheritedFolderRule> = folder_config
                    .map_or(Vec::new(), |folder_config| {
                        folder_config
                            .folder_rules
                            .iter()
                            .filter_map(|rule| match rule.non_recursive {
                                true => None,
                                false => {
                                    Some(InheritedFolderRule { rule: rule.clone() })
                                }
                            })
                            .collect()
                    });

                let sub_folder_inherited_folders_rules =
                    [inherited_folders_rules.clone(), parent_folder_rules].concat();

                if let Err(extra_errors) = check_folder_childs(
                    sub_folder,
                    sub_folder_cfg,
                    parent_path,
                    sub_folder_inherited_files_rules,
                    sub_folder_inherited_folders_rules,
                    used_files_deps_info,
                ) {
                    errors.extend(extra_errors);
                }
            }
        }
    }

    for folder_missing in folders_missing_check {
        errors.push(format!(
            "Folder '/{}' is missing in folder '{}'",
            folder_missing, folder_path
        ));
    }

    if !errors.is_empty() {
        Err(errors)
    } else {
        Ok(())
    }
}

pub fn check_root_folder(
    config: &Config,
    folder: &Folder,
    used_files_deps_info: &mut UsedFilesDepsInfo,
) -> Result<(), Vec<String>> {
    check_folder_childs(
        folder,
        Some(&config.root_folder),
        String::from("."),
        Vec::new(),
        Vec::new(),
        used_files_deps_info,
    )
}

mod checks;
#[cfg(test)]
mod tests;
