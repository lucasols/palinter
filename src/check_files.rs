use std::collections::HashSet;

use crate::{
    expect_checks::*,
    internal_config::{AnyOr, Config, FileConditions, FileExpect, FileRule, FolderConfig},
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

fn file_pass_expected(file: &File, expected: &AnyOr<Vec<FileExpect>>) -> Result<(), String> {
    match expected {
        AnyOr::Any => Ok(()),
        AnyOr::Or(expected) => {
            for expect in expected {
                if let Some(file_name_case_is) = &expect.name_case_is {
                    name_case_is(&file.name, file_name_case_is)?;
                }
            }

            Ok(())
        }
    }
}

fn check_folder_childs(
    config: &Config,
    folder: &Folder,
    folder_config: &FolderConfig,
    folder_path: String,
) -> Result<(), Vec<String>> {
    let mut errors: Vec<String> = Vec::new();

    let mut folders_missing_check: HashSet<String> = folder_config
        .sub_folders_config
        .keys()
        .map(|key| key.to_string())
        .collect();

    for child in &folder.childs {
        match child {
            Child::FileChild(file) => {
                let mut file_matched_at_least_once = false;

                let file_error_prefix = format!(
                    "File '{}/{}.{}' error: ",
                    folder_path, file.name, file.extension
                );

                for rule in &config.global_files_rules {
                    let file_matches = file_matches_condition(file, &rule.conditions);

                    if file_matches {
                        file_matched_at_least_once = true;

                        if let Err(error) = file_pass_expected(file, &rule.expect) {
                            errors.push(format!("{}{}", file_error_prefix, error));
                        }
                    }
                }

                for rule in &folder_config.file_rules {
                    let file_matches = file_matches_condition(file, &rule.conditions);

                    if file_matches {
                        file_matched_at_least_once = true;

                        if let Err(error) = file_pass_expected(file, &rule.expect) {
                            errors.push(format!("{}{}", file_error_prefix, error));
                        }
                    }
                }

                if !file_matched_at_least_once {
                    errors.push(format!(
                        "File '{}.{}' is not expected in folder '{}'",
                        file.name, file.extension, folder_path
                    ));
                }
            }
            Child::Folder(sub_folder) => {
                let sub_folder_config = folder_config.sub_folders_config.get(&sub_folder.name);

                match sub_folder_config {
                    None => {
                        errors.push(format!(
                            "Folder '{}' is not expected in folder '{}'",
                            sub_folder.name, folder_path
                        ));
                    }
                    Some(sub_folder_cfg) => {
                        folders_missing_check.remove(&sub_folder.name);

                        let parent_path = if folder_path.is_empty() {
                            sub_folder.name.clone()
                        } else {
                            format!("{}{}", folder_path, sub_folder.name)
                        };

                        if let Err(extra_errors) =
                            check_folder_childs(config, sub_folder, sub_folder_cfg, parent_path)
                        {
                            errors.extend(extra_errors);
                        }
                    }
                }
            }
        }
    }

    for folder_missing in folders_missing_check {
        errors.push(format!(
            "Folder '{}' is missing in folder '{}'",
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
    check_folder_childs(config, folder, &config.root_folder, String::from("."))
}

#[cfg(test)]
mod tests;
