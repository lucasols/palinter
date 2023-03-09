use std::collections::HashSet;

use crate::{
    expect_checks::*,
    internal_config::{
        AnyOr, Config, FileConditions, FileExpect, FileRule, FolderConditions, FolderConfig,
        FolderExpect, FolderRule,
    },
};

#[derive(Debug)]
struct File {
    name: String,
    content: String,
    extension: String,
}

#[derive(Debug)]
enum Child {
    FileChild(File),
    Folder(Folder),
}

#[derive(Debug)]
pub struct Folder {
    name: String,
    childs: Vec<Child>,
}

fn file_matches_condition(file: &File, conditions: &AnyOr<FileConditions>) -> bool {
    match conditions {
        AnyOr::Any => true,
        AnyOr::Or(conditions) => {
            if let Some(extensions) = &conditions.has_extension {
                if !extensions.contains(&file.extension) {
                    return false;
                }
            }

            true
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
                Err(format!("{} | {}", expect_error_msg, error))
            } else {
                Err(error)
            }
        }
    }
}

fn file_pass_expected(file: &File, expected: &AnyOr<Vec<FileExpect>>) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    let mut check_result = |result: Result<(), String>, expect_error_msg: &Option<String>| {
        if let Err(error) = append_expect_error(result, expect_error_msg) {
            errors.push(error);
        }
    };

    if let AnyOr::Or(expected) = expected {
        for expect in expected {
            if let Some(file_name_case_is) = &expect.name_case_is {
                check_result(
                    name_case_is(&file.name, file_name_case_is),
                    &expect.error_msg,
                );
            }

            if let Some(file_extension_is) = &expect.extension_is {
                check_result(
                    extension_is(&file.extension, file_extension_is),
                    &expect.error_msg,
                );
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn folder_matches_condition(folder: &Folder, conditions: &AnyOr<FolderConditions>) -> bool {
    match conditions {
        AnyOr::Any => true,
        AnyOr::Or(_) => true,
    }
}

fn folder_pass_expected(
    folder: &Folder,
    expected: &AnyOr<Vec<FolderExpect>>,
) -> Result<(), String> {
    match expected {
        AnyOr::Any => Ok(()),
        AnyOr::Or(expected) => {
            for expect in expected {
                if let Some(file_name_case_is) = &expect.name_case_is {
                    append_expect_error(
                        name_case_is(&folder.name, file_name_case_is),
                        &expect.error_msg,
                    )?;
                }
            }

            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
struct InheritedFileRule {
    from_path: String,
    rule: FileRule,
}

#[derive(Debug, Clone)]
struct InheritedFolderRule {
    from_path: String,
    rule: FolderRule,
}

pub fn normalize_folder_config_name(name: &String) -> String {
    if name == "." {
        name.to_owned()
    } else {
        name.strip_prefix('/').unwrap().to_string()
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
    config: &Config,
    folder: &Folder,
    folder_config: Option<&FolderConfig>,
    folder_path: String,
    inherited_files_rules: Vec<InheritedFileRule>,
    inherited_folders_rules: Vec<InheritedFolderRule>,
) -> Result<(), Vec<String>> {
    let mut errors: Vec<String> = Vec::new();

    let mut folders_missing_check = folder_config
        .map(|folder_config| {
            folder_config
                .sub_folders_config
                .keys()
                .map(normalize_folder_config_name)
                .collect::<HashSet<String>>()
        })
        .unwrap_or_default();

    for child in &folder.childs {
        match child {
            Child::FileChild(file) => {
                let mut file_matched_at_least_once = false;

                let file_error_prefix = format!(
                    "File '{}/{}.{}' error: ",
                    folder_path, file.name, file.extension
                );

                let mut check_file_rule = |rule: &FileRule| {
                    let file_matches = file_matches_condition(file, &rule.conditions);

                    if file_matches {
                        file_matched_at_least_once = true;

                        if let Err(expect_errors) = file_pass_expected(file, &rule.expect) {
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

                for rule in &config.global_files_rules {
                    check_file_rule(rule)
                }

                if let Some(folder_config) = folder_config {
                    for rule in &folder_config.file_rules {
                        check_file_rule(rule)
                    }
                }

                for inherited_rule in &inherited_files_rules {
                    check_file_rule(&inherited_rule.rule)
                }

                if !file_matched_at_least_once {
                    errors.push(format!(
                        "File '{}.{}' is not expected in folder '{}'",
                        file.name, file.extension, folder_path
                    ));
                }
            }
            Child::Folder(sub_folder) => {
                let folder_error_prefix =
                    format!("Folder '{}/{}' error: ", folder_path, sub_folder.name);

                let mut folder_matched_at_least_once = false;

                let mut check_folder_rule = |rule: &FolderRule| {
                    let folder_matches = folder_matches_condition(sub_folder, &rule.conditions);

                    if folder_matches {
                        folders_missing_check.remove(&sub_folder.name);

                        folder_matched_at_least_once = true;

                        if let Err(error) = folder_pass_expected(sub_folder, &rule.expect) {
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

                for rule in &config.global_folders_rules {
                    check_folder_rule(rule);
                }

                if let Some(folder_config) = folder_config {
                    for rule in &folder_config.folder_rules {
                        check_folder_rule(rule);
                    }
                }

                for inheridte_rule in &inherited_folders_rules {
                    check_folder_rule(&inheridte_rule.rule);
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
                    folders_missing_check.remove(&sub_folder.name);
                } else if !folder_matched_at_least_once {
                    errors.push(format!(
                        "Folder '/{}' is not expected in folder '{}'",
                        sub_folder.name, folder_path
                    ));
                }

                let parent_file_rules: Vec<InheritedFileRule> =
                    folder_config.map_or(Vec::new(), |folder_config| {
                        folder_config
                            .file_rules
                            .iter()
                            .filter_map(|rule| match rule.non_recursive {
                                true => None,
                                false => Some(InheritedFileRule {
                                    from_path: folder_path.clone(),
                                    rule: rule.clone(),
                                }),
                            })
                            .collect()
                    });

                let sub_folder_inherited_files_rules =
                    [inherited_files_rules.clone(), parent_file_rules].concat();

                let parent_folder_rules: Vec<InheritedFolderRule> =
                    folder_config.map_or(Vec::new(), |folder_config| {
                        folder_config
                            .folder_rules
                            .iter()
                            .filter_map(|rule| match rule.non_recursive {
                                true => None,
                                false => Some(InheritedFolderRule {
                                    from_path: folder_path.clone(),
                                    rule: rule.clone(),
                                }),
                            })
                            .collect()
                    });

                let sub_folder_inherited_folders_rules =
                    [inherited_folders_rules.clone(), parent_folder_rules].concat();

                if let Err(extra_errors) = check_folder_childs(
                    config,
                    sub_folder,
                    sub_folder_cfg,
                    parent_path,
                    sub_folder_inherited_files_rules,
                    sub_folder_inherited_folders_rules,
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

pub fn check_root_folder(config: &Config, folder: &Folder) -> Result<(), Vec<String>> {
    check_folder_childs(
        config,
        folder,
        Some(&config.root_folder),
        String::from("."),
        Vec::new(),
        Vec::new(),
    )
}

#[cfg(test)]
mod tests;
