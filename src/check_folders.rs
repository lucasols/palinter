use colored::Colorize;
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::{
    analyze_ts_deps::ts_checks::{
        check_ts_have_imports, check_ts_not_have_circular_deps,
        check_ts_not_have_deps_from, check_ts_not_have_deps_outside,
        check_ts_not_have_direct_circular_deps, check_ts_not_have_imports,
        check_ts_not_have_unused_exports, check_ts_not_have_used_exports_outside,
    },
    internal_config::{
        AnyNoneOr, AnyOr, Config, ErrorMsgVars, FileConditions, FileExpect,
        FileRule, FolderConditions, FolderConfig, FolderExpect, FolderRule,
        OneOfBlocks,
    },
    load_folder_structure::{File, Folder, FolderChild},
    utils::clone_extend_vec,
};

use self::checks::{
    check_content, check_content_not_matches, check_file_is_not_empty,
    check_folder_min_children, check_negated_path_pattern,
    check_negated_root_files_has_pattern, check_path_pattern,
    check_root_files_find_pattern, check_root_files_has_pattern,
    expand_to_capture_case_variation, extension_is, has_sibling_file, name_case_is,
    path_pattern_match, Capture,
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

            if let Some(content_matches) = &conditions.has_content {
                if check_content(&file.content, content_matches, &[], false).is_err()
                {
                    return None;
                }
            }

            if let Some(content_matches) = &conditions.has_any_content {
                if check_content(
                    &file.content,
                    content_matches,
                    &has_name_captures,
                    true,
                )
                .is_err()
                {
                    return None;
                }
            }

            if let Some(content_matches) = &conditions.not_has_content {
                if check_content_not_matches(
                    &file.content,
                    content_matches,
                    &has_name_captures,
                )
                .is_err()
                {
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
    error_msg_vars: &ErrorMsgVars,
) -> Result<(), String> {
    match result {
        Ok(_) => Ok(()),
        Err(error) => {
            if let Some(expect_error_msg) = expect_error_msg {
                let custom_err_msg =
                    replace_error_msg_vars(error_msg_vars, expect_error_msg);

                Err(format!("{}\n   | {}", custom_err_msg, error))
            } else {
                Err(error)
            }
        }
    }
}

fn replace_error_msg_vars(
    error_msg_vars: &Option<BTreeMap<String, String>>,
    expect_error_msg: &String,
) -> String {
    if let Some(vars) = error_msg_vars {
        let mut err_msg = expect_error_msg.clone();

        for (var, value) in vars {
            err_msg = err_msg.replace(&format!("${{{}}}", var), value);
        }

        err_msg
    } else {
        expect_error_msg.to_string()
    }
}

fn append_error_to_vec(
    result: Result<(), String>,
    expect_error_msg: &Option<String>,
    error_msg_vars: &ErrorMsgVars,
) -> Result<(), Vec<String>> {
    let result = append_expect_error(result, expect_error_msg, error_msg_vars);

    match result {
        Ok(_) => Ok(()),
        Err(error) => Err(vec![error]),
    }
}

fn check_file_expect(
    file: &File,
    expected: &AnyNoneOr<Vec<FileExpect>>,
    folder: &Folder,
    conditions_result: &ConditionsResult,
    context_conditions: &[Capture],
    error_msg_vars: &ErrorMsgVars,
) -> Result<(), Vec<String>> {
    if let AnyNoneOr::None = expected {
        return Err(vec!["File is not expected".to_string()]);
    }

    let mut errors = Vec::new();

    let mut check_result =
        |result: Result<(), String>, expect_error_msg: &Option<String>| {
            if let Err(error) =
                append_expect_error(result, expect_error_msg, error_msg_vars)
            {
                errors.push(error);
            }
        };

    if let AnyNoneOr::Or(expected) = expected {
        let captures =
            clone_extend_vec(&conditions_result.captures, context_conditions);

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
                    has_sibling_file(sibling_file_pattern, folder, &captures),
                    &expect.error_msg,
                );
            }

            if let Some(content_matches) = &expect.content_matches {
                pass_some_expect = true;
                check_result(
                    check_content(&file.content, content_matches, &captures, false),
                    &expect.error_msg,
                );
            }

            if let Some(content_matches_some) = &expect.content_matches_some {
                pass_some_expect = true;
                check_result(
                    check_content(
                        &file.content,
                        content_matches_some,
                        &captures,
                        true,
                    ),
                    &expect.error_msg,
                );
            }

            if let Some(name_is) = &expect.name_is {
                pass_some_expect = true;
                check_result(
                    check_path_pattern(&file.name_with_ext, name_is, &captures),
                    &expect.error_msg,
                );
            }

            if let Some(name_is_not) = &expect.name_is_not {
                pass_some_expect = true;
                check_result(
                    check_negated_path_pattern(
                        &file.name_with_ext,
                        name_is_not,
                        &captures,
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
                        &captures,
                    ),
                    &expect.error_msg,
                );
            }

            if expect.is_not_empty {
                pass_some_expect = true;
                check_result(check_file_is_not_empty(file), &expect.error_msg);
            }

            if let Some(ts_expect) = &expect.ts {
                if ts_expect.not_have_unused_exports {
                    pass_some_expect = true;
                    check_result(
                        check_ts_not_have_unused_exports(file),
                        &expect.error_msg,
                    );
                }

                if ts_expect.not_have_circular_deps {
                    pass_some_expect = true;
                    check_result(
                        check_ts_not_have_circular_deps(file),
                        &expect.error_msg,
                    );
                }

                if let Some(disallow) = &ts_expect.not_have_deps_from {
                    pass_some_expect = true;
                    check_result(
                        check_ts_not_have_deps_from(file, disallow),
                        &expect.error_msg,
                    );
                }

                if let Some(allowed) = &ts_expect.not_have_deps_outside {
                    pass_some_expect = true;
                    check_result(
                        check_ts_not_have_deps_outside(file, allowed),
                        &expect.error_msg,
                    );
                }

                if let Some(allowed) = &ts_expect.not_have_exports_used_outside {
                    pass_some_expect = true;
                    check_result(
                        check_ts_not_have_used_exports_outside(file, allowed),
                        &expect.error_msg,
                    );
                }

                if ts_expect.not_have_direct_circular_deps {
                    pass_some_expect = true;
                    check_result(
                        check_ts_not_have_direct_circular_deps(file),
                        &expect.error_msg,
                    );
                }

                if let Some(imports) = &ts_expect.have_imports {
                    pass_some_expect = true;
                    check_result(
                        check_ts_have_imports(file, imports),
                        &expect.error_msg,
                    );
                }

                if let Some(not_have_imports) = &ts_expect.not_have_imports {
                    pass_some_expect = true;
                    check_result(
                        check_ts_not_have_imports(file, not_have_imports),
                        &expect.error_msg,
                    );
                }
            }

            if cfg!(debug_assertions) && !pass_some_expect {
                panic!("Unexpected expect {:#?}", expect);
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

#[derive(Debug)]
pub struct Problems {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug)]
enum FolderExpectError {
    Errors(Vec<String>),
    ChildProblems(Problems),
}

#[allow(clippy::too_many_arguments)]
fn check_folder_expected(
    allow_warnings: bool,
    folder: &Folder,
    expected: &AnyNoneOr<Vec<FolderExpect>>,
    conditions_result: &ConditionsResult,
    folder_path: &str,
    inherited_files_rules: &[InheritedFileRule],
    inherited_folders_rules: &[InheritedFolderRule],
    context_conditions: &[Capture],
    error_msg_vars: &ErrorMsgVars,
    is_test_config: bool,
) -> Result<(), FolderExpectError> {
    match expected {
        AnyNoneOr::None => Err(FolderExpectError::Errors(vec![
            "Folder is not expected".to_string(),
        ])),
        AnyNoneOr::Any => Ok(()),
        AnyNoneOr::Or(expected) => {
            let mut pass_some_expect = false;

            let captures =
                clone_extend_vec(&conditions_result.captures, context_conditions);

            for expect in expected {
                if let Some(file_name_case_is) = &expect.name_case_is {
                    pass_some_expect = true;
                    append_error_to_vec(
                        name_case_is(&folder.name, file_name_case_is),
                        &expect.error_msg,
                        error_msg_vars,
                    )
                    .map_err(FolderExpectError::Errors)?;
                }

                if let Some(name_is) = &expect.name_is {
                    pass_some_expect = true;
                    append_error_to_vec(
                        check_path_pattern(&folder.name, name_is, &captures),
                        &expect.error_msg,
                        error_msg_vars,
                    )
                    .map_err(FolderExpectError::Errors)?;
                }

                if let Some(name_is_not) = &expect.name_is_not {
                    pass_some_expect = true;
                    append_error_to_vec(
                        check_negated_path_pattern(
                            &folder.name,
                            name_is_not,
                            &captures,
                        ),
                        &expect.error_msg,
                        error_msg_vars,
                    )
                    .map_err(FolderExpectError::Errors)?;
                }

                if let Some(root_files_has) = &expect.root_files_has {
                    pass_some_expect = true;
                    append_error_to_vec(
                        check_root_files_has_pattern(
                            folder,
                            root_files_has,
                            &captures,
                        )
                        .map(|_| ()),
                        &expect.error_msg,
                        error_msg_vars,
                    )
                    .map_err(FolderExpectError::Errors)?;
                }

                if let Some(root_files_has_not) = &expect.root_files_has_not {
                    pass_some_expect = true;
                    append_error_to_vec(
                        check_negated_root_files_has_pattern(
                            folder,
                            root_files_has_not,
                            &captures,
                        ),
                        &expect.error_msg,
                        error_msg_vars,
                    )
                    .map_err(FolderExpectError::Errors)?;
                }

                if let Some(min_children) = &expect.have_min_children {
                    pass_some_expect = true;
                    append_error_to_vec(
                        check_folder_min_children(folder, *min_children),
                        &expect.error_msg,
                        error_msg_vars,
                    )
                    .map_err(FolderExpectError::Errors)?;
                }

                if let Some((folder_rules, file_rules)) = &expect.child_rules {
                    pass_some_expect = true;

                    check_folder_children(
                        allow_warnings,
                        folder,
                        Some(&FolderConfig {
                            file_rules: file_rules.clone(),
                            folder_rules: folder_rules.clone(),
                            allow_unexpected_files: false,
                            allow_unexpected_folders: false,
                            sub_folders_config: HashMap::default(),
                            optional: false,
                            one_of_blocks: OneOfBlocks::default(),
                            unexpected_files_error_msg: None,
                            unexpected_folders_error_msg: None,
                            unexpected_error_msg: None,
                            append_error_msg: None,
                        }),
                        folder_path.to_string(),
                        inherited_files_rules.to_vec(),
                        inherited_folders_rules.to_vec(),
                        expand_to_capture_case_variation(
                            "context_folder",
                            folder.name.clone(),
                        ),
                        error_msg_vars,
                        is_test_config,
                        false,
                        false,
                        false,
                    )
                    .map_err(FolderExpectError::ChildProblems)?;
                }

                if cfg!(debug_assertions) && !pass_some_expect {
                    panic!("Unexpected expect {:#?}", expect);
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

fn check_folder_children(
    allow_warnings: bool,
    folder: &Folder,
    folder_config: Option<&FolderConfig>,
    folder_path: String,
    inherited_files_rules: Vec<InheritedFileRule>,
    inherited_folders_rules: Vec<InheritedFolderRule>,
    context_conditions: Vec<Capture>,
    error_msg_vars: &ErrorMsgVars,
    is_test_config: bool,
    inherited_select_all_children: bool,
    inherited_allow_unconfigured_files: bool,
    inherited_allow_unconfigured_folders: bool,
) -> Result<(), Problems> {
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    let append_error = folder_config
        .and_then(|fc| {
            fc.append_error_msg
                .as_ref()
                .map(|msg| replace_error_msg_vars(error_msg_vars, msg))
        })
        .map(|append_err| format!("\n   | {}", &append_err.dimmed()))
        .unwrap_or_default();

    let allow_unconfigured_folders = folder_config
        .map_or(inherited_allow_unconfigured_folders, |folder_config| {
            folder_config.allow_unexpected_folders
        });
    let allow_unconfigured_files = folder_config
        .map_or(inherited_allow_unconfigured_files, |folder_config| {
            folder_config.allow_unexpected_files
        });

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

    for child in &folder.children {
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

                        if is_test_config && rule.ignore_in_config_tests {
                            return;
                        }

                        if let Err(expect_errors) = check_file_expect(
                            file,
                            &rule.expect,
                            folder,
                            &conditions_result,
                            &context_conditions,
                            error_msg_vars,
                        ) {
                            for error in expect_errors {
                                let problem_vec =
                                    if allow_warnings && rule.is_warning {
                                        &mut warnings
                                    } else {
                                        &mut errors
                                    };

                                problem_vec.push(format!(
                                    "{}{}{}",
                                    file_error_prefix,
                                    if let Some(custom_error) = &rule.error_msg {
                                        format!("{}\n   | {}", custom_error, error)
                                    } else {
                                        error
                                    },
                                    append_error
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

                                if check_file_expect(
                                    file,
                                    &rule.expect,
                                    folder,
                                    &conditions_result,
                                    &context_conditions,
                                    error_msg_vars,
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
                                "{}{}{}",
                                file_error_prefix, one_of.error_msg, append_error
                            ));
                        }
                    }
                }

                if !file_touched && !allow_unconfigured_files {
                    errors.push(format!(
                        "File {} is not expected in folder {}{}{}",
                        file.name_with_ext.bright_yellow(),
                        folder_path.bright_red(),
                        folder_config
                            .and_then(|cfg| cfg
                                .unexpected_files_error_msg
                                .as_ref()
                                .or(cfg.unexpected_error_msg.as_ref()))
                            .map(|msg| format!("\n   | {}", msg))
                            .unwrap_or_default(),
                        append_error
                    ));
                }
            }
            FolderChild::Folder(sub_folder) => {
                let folder_error_prefix = format!(
                    "Folder {}\n • ",
                    format!("{}/{}:", folder_path, sub_folder.name).bright_red()
                );

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

                let mut folder_touched = false;
                let mut folder_has_error = false;
                let mut children_was_checked = false;

                let mut check_folder_rule = |rule: &FolderRule| {
                    let folder_matches =
                        folder_matches_condition(sub_folder, &rule.conditions);

                    if let Some(conditions_result) = folder_matches {
                        folders_missing_check.remove(&sub_folder.name);

                        if !rule.not_touch {
                            folder_touched = true;
                        }

                        if let AnyNoneOr::Or(expect_rules) = &rule.expect {
                            if expect_rules
                                .iter()
                                .any(|rule| rule.child_rules.is_some())
                            {
                                children_was_checked = true;
                            }
                        }

                        if let Err(folder_expect_error) = check_folder_expected(
                            allow_warnings,
                            sub_folder,
                            &rule.expect,
                            &conditions_result,
                            &folder_path,
                            &sub_folder_inherited_files_rules,
                            &sub_folder_inherited_folders_rules,
                            &context_conditions,
                            error_msg_vars,
                            is_test_config,
                        ) {
                            match folder_expect_error {
                                FolderExpectError::Errors(problems_found) => {
                                    if !problems_found.is_empty() {
                                        folder_has_error = true;

                                        let problem_vec =
                                            if rule.is_warning && allow_warnings {
                                                &mut warnings
                                            } else {
                                                &mut errors
                                            };

                                        push_to_folder_problem_vec(
                                            problems_found,
                                            problem_vec,
                                            &folder_error_prefix,
                                            rule,
                                            &append_error,
                                        );
                                    }
                                }
                                FolderExpectError::ChildProblems(child_problems) => {
                                    push_to_folder_problem_vec(
                                        child_problems.errors,
                                        &mut errors,
                                        &folder_error_prefix,
                                        rule,
                                        &append_error,
                                    );

                                    push_to_folder_problem_vec(
                                        child_problems.warnings,
                                        &mut warnings,
                                        &folder_error_prefix,
                                        rule,
                                        &append_error,
                                    );
                                }
                            }
                        }
                    }
                };

                if let Some(folder_config) = folder_config {
                    for rule in &folder_config.folder_rules {
                        check_folder_rule(rule);
                    }
                }

                for inherited_rule in &inherited_folders_rules {
                    check_folder_rule(&inherited_rule.rule);
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

                let mut folder_is_not_expected = false;

                if sub_folder_cfg.is_some() {
                    folders_missing_check.remove(&sub_folder.name);
                } else if !folder_touched && !allow_unconfigured_folders {
                    folder_is_not_expected = true;
                    errors.push(format!(
                        "Folder {} is not expected in folder {}{}{}",
                        format!("/{}", sub_folder.name).bright_red(),
                        folder_path.bright_red(),
                        folder_config
                            .and_then(|cfg| cfg
                                .unexpected_folders_error_msg
                                .as_ref()
                                .or(cfg.unexpected_error_msg.as_ref()))
                            .map(|msg| format!("\n   | {}", msg))
                            .unwrap_or_default(),
                        append_error
                    ));
                }

                let new_sub_folder_cfg =
                    sub_folder_cfg.map(|sub_folder_cfg| FolderConfig {
                        append_error_msg: sub_folder_cfg
                            .append_error_msg
                            .clone()
                            .or(folder_config
                                .and_then(|cfg| cfg.append_error_msg.clone())),
                        ..sub_folder_cfg.clone()
                    });

                if !folder_is_not_expected {
                    if let Err(Problems {
                        errors: extra_errors,
                        warnings: extra_warnings,
                    }) = check_folder_children(
                        allow_warnings,
                        sub_folder,
                        new_sub_folder_cfg.as_ref(),
                        parent_path,
                        sub_folder_inherited_files_rules,
                        sub_folder_inherited_folders_rules,
                        Vec::new(),
                        error_msg_vars,
                        is_test_config,
                        true,
                        if inherited_select_all_children {
                            allow_unconfigured_files
                        } else {
                            false
                        },
                        if inherited_select_all_children {
                            allow_unconfigured_folders
                        } else {
                            false
                        },
                    ) {
                        errors.extend(extra_errors);
                        warnings.extend(extra_warnings);
                    }
                }
            }
        }
    }

    for folder_missing in folders_missing_check {
        errors.push(format!(
            "Folder '/{}' is missing in folder '{}'{}",
            folder_missing, folder_path, append_error
        ));
    }

    if !errors.is_empty() || !warnings.is_empty() {
        Err(Problems { errors, warnings })
    } else {
        Ok(())
    }
}

fn push_to_folder_problem_vec(
    problems_found: Vec<String>,
    problem_vec: &mut Vec<String>,
    folder_error_prefix: &String,
    rule: &FolderRule,
    append_error: &String,
) {
    for error in problems_found {
        problem_vec.push(format!(
            "{}{}{}",
            folder_error_prefix,
            if let Some(custom_error) = &rule.error_msg {
                format!("{}\n   | {}", custom_error, error)
            } else {
                error
            },
            append_error
        ));
    }
}

pub fn check_root_folder(
    config: &Config,
    folder: &Folder,
    is_test_config: bool,
    allow_warnings: bool,
) -> Result<(), Problems> {
    check_folder_children(
        allow_warnings || config.allow_warnings,
        folder,
        Some(&config.root_folder),
        String::from("."),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        &config.error_msg_vars,
        is_test_config,
        false,
        false,
        false,
    )
}

mod checks;
#[cfg(test)]
mod tests;
