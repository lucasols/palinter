use colored::Colorize;
use regex::Regex;
use std::{
    collections::{BTreeMap, HashSet},
    hash::Hash,
    path::{Path, PathBuf},
};

use crate::{
    analyze_ts_deps::{_setup_test, load_used_project_files_deps_info_from_cfg},
    check_folders::{check_root_folder, normalize_folder_config_name, Problems},
    internal_config::get_config,
    load_folder_structure::{File, Folder, FolderChild},
    parse_config_file,
    test_utils::TEST_MUTEX,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
struct TestCase {
    file_name: String,
    file_content: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
enum ParsedStructureChild {
    Folder(ParsedFolder),
    File(String),
}

#[derive(Deserialize, Serialize, Debug)]
struct ParsedFolder {
    #[serde(flatten)]
    childs: BTreeMap<String, ParsedStructureChild>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
enum ExpectedErrors {
    Single(bool),
    Multiple(Vec<String>),
}

#[derive(Deserialize, Serialize, Debug)]
struct ParsedProjectYaml {
    only: Option<bool>,
    structure: ParsedFolder,
    expected_errors: ExpectedErrors,
    files: Option<BTreeMap<String, String>>,
}

#[derive(Debug)]
struct Project {
    only: bool,
    structure: Folder,
    expected_errors: Option<Vec<String>>,
}

struct UpdateExpectedErrors {
    test_case_path: String,
    project_index: usize,
    expected_errors: Vec<String>,
}

pub fn test_config(
    test_cases_dir: &PathBuf,
    config_file: &PathBuf,
    update_expected_errors: bool,
) -> Result<String, String> {
    let files_content = get_test_cases(test_cases_dir).map_err(|error| {
        format!("Error getting test cases from folder: {}", error)
    })?;

    let parsed_config = parse_config_file(config_file)?;

    let config = get_config(&parsed_config)
        .map_err(|error| format!("Error parsing config file: {}", error))?;

    let mut test_summary: String;

    let mut ignored_tests = 0;

    let only_files_content_to_test = files_content
        .iter()
        .filter(|&TestCase { file_content, .. }| file_content.starts_with("only"))
        .cloned()
        .collect::<Vec<TestCase>>();

    let files_content_to_test = if only_files_content_to_test.is_empty() {
        let to_test: Vec<TestCase> = files_content
            .iter()
            .filter(|&TestCase { file_content, .. }| {
                if file_content.starts_with("ignore") {
                    ignored_tests += 1;

                    return false;
                }

                true
            })
            .cloned()
            .collect();

        test_summary = format!("\n🟩 Running {} test cases\n", to_test.len());

        to_test
    } else {
        test_summary = format!(
            "\n🟧 Running only test cases with 'only' prefix -> {}\n",
            only_files_content_to_test
                .clone()
                .into_iter()
                .map(|TestCase { file_name, .. }| file_name)
                .collect::<Vec<String>>()
                .join(", ")
        );

        only_files_content_to_test
    };

    if ignored_tests > 0 {
        test_summary = format!(
            "{}\n🟧 Ignored {} test cases\n",
            test_summary, ignored_tests
        );
    }

    let mut test_errors: Vec<String> = vec![];
    let mut expected_errors_to_update: Vec<UpdateExpectedErrors> = vec![];

    for TestCase {
        file_name,
        file_content,
    } in files_content_to_test
    {
        let projects = extract_projects_from_file_content(file_content);

        match projects {
            Err(err) => {
                test_errors
                    .push(format!("❌ Error parsing file '{}': {}", file_name, err));
            }
            Ok(projects) => {
                let some_project_has_only =
                    projects.iter().any(|project| project.only);

                for (i, project) in projects
                    .iter()
                    .filter(|project| {
                        if some_project_has_only {
                            project.only
                        } else {
                            true
                        }
                    })
                    .enumerate()
                {
                    colored::control::set_override(false);
                    _setup_test();
                    let _guard = TEST_MUTEX.lock().unwrap();

                    match load_used_project_files_deps_info_from_cfg(
                        &config,
                        &project.structure,
                        Path::new("."),
                    ) {
                        Ok(used_files_deps_info) => used_files_deps_info,
                        Err(error) => {
                            test_errors.push(format!(
                                "❌ Test case '{}': Project {}: {}",
                                file_name.blue(),
                                i + 1,
                                error
                            ));

                            continue;
                        }
                    };

                    let result =
                        check_root_folder(&config, &project.structure, true, false);

                    colored::control::unset_override();

                    let test_case = format!(
                        "❌ Test case '{}' - project {}:",
                        file_name.blue(),
                        i + 1
                    );

                    match &project.expected_errors {
                        Some(expected_errors) => {
                            if let Err(Problems { errors, .. }) = result {
                                let collected = &expected_errors
                                    .iter()
                                    .map(|err| err.trim().to_string())
                                    .collect::<Vec<String>>();

                                if !do_vecs_match(&errors, collected) {
                                    if update_expected_errors {
                                        expected_errors_to_update.push(
                                            UpdateExpectedErrors {
                                                test_case_path: format!(
                                                    "{}",
                                                    test_cases_dir
                                                        .join(&file_name)
                                                        .display()
                                                ),
                                                project_index: i,
                                                expected_errors: sort_vector(
                                                    &errors,
                                                ),
                                            },
                                        );

                                        test_summary.push_str(
                                            format!(
                                                "\n\n🟧 Updated expected errors for test case '{}'\n",
                                                file_name
                                            )
                                            .as_str(),
                                        );
                                    } else {
                                        test_errors.push(format!(
                                            "{}\n\
                                                    Expected errors: {:#?}\n\
                                                    But got:         {:#?}",
                                            test_case,
                                            sort_vector(collected),
                                            sort_vector(&errors)
                                        ));
                                    }
                                }
                            } else {
                                test_errors.push(format!(
                                    "{} Expected errors but got Ok",
                                    test_case
                                ));
                            }
                        }
                        None => {
                            if let Err(error) = result {
                                test_errors.push(format!(
                                    "{} Expected Ok but got errors: {:#?}",
                                    test_case, error
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    if update_expected_errors {
        apply_expected_errors_updates(&expected_errors_to_update)?;
    }

    if !test_errors.is_empty() {
        Err(format!(
            "\n\n{}\n\n{}\n",
            test_errors.join("\n\n"),
            test_summary
        ))
    } else {
        Ok(format!("{}\n", test_summary))
    }
}

fn apply_expected_errors_updates(
    expected_errors_to_update: &[UpdateExpectedErrors],
) -> Result<(), String> {
    for UpdateExpectedErrors {
        test_case_path,
        project_index,
        expected_errors,
    } in expected_errors_to_update
    {
        let test_case_content =
            std::fs::read_to_string(test_case_path).map_err(|err| {
                format!("Error reading test case '{}': {}", test_case_path, err)
            })?;

        let projects_caputres = get_projects_capture(&test_case_content);

        let mut new_test_case_content = test_case_content.to_string();

        for (i, project_capture) in projects_caputres.into_iter().enumerate() {
            if i == *project_index {
                let project_yaml = project_capture;

                // replace string in range from 'expected_errors:' to the end with new expected errors

                let mut new_project_yaml = project_yaml.to_string();

                let expected_errors_index =
                    new_project_yaml.find("expected_errors:").ok_or_else(|| {
                        format!(
                            "Missing 'expected_errors:' section in '{}'",
                            test_case_path
                        )
                    })?;

                new_project_yaml.replace_range(
                    expected_errors_index..,
                    format!(
                        "expected_errors:\n{}",
                        expected_errors
                            .iter()
                            .map(|err| {
                                let new_err_with_balenced_new_lines = err
                                    .replace("\n   |", "\n       |")
                                    .replace("\n •", "\n     •");

                                format!(
                                    "  - |\n    {}",
                                    new_err_with_balenced_new_lines
                                )
                            })
                            .collect::<Vec<String>>()
                            .join("\n")
                    )
                    .as_str(),
                );

                new_test_case_content = new_test_case_content
                    .replace(project_yaml, new_project_yaml.as_str());
            }
        }

        std::fs::write(test_case_path, new_test_case_content).map_err(|err| {
            format!(
                "Error writing updated test case '{}': {}",
                test_case_path, err
            )
        })?;
    }

    Ok(())
}

fn extract_projects_from_file_content(
    test_case_content: String,
) -> Result<Vec<Project>, String> {
    let mut projects: Vec<Project> = Vec::new();

    let projects_captures = get_projects_capture(&test_case_content);

    for (i, project_capture) in projects_captures.into_iter().enumerate() {
        let project_yaml = project_capture;

        let project = parse_project_yaml(project_yaml.to_string());

        match project {
            Ok(project) => projects.push(project),
            Err(error) => {
                return Err(format!(
                    "Error parsing project {} yaml: {}",
                    i + 1,
                    error
                ))
            }
        }
    }

    if projects.is_empty() {
        return Err("No projects found".to_string());
    }

    Ok(projects)
}

fn get_projects_capture(test_case_content: &str) -> Vec<&str> {
    let projects_regex = Regex::new(r"```yaml\n([\S\s]+?)\n```").unwrap();

    let captures = projects_regex.captures_iter(test_case_content);

    let mut matches = Vec::new();
    for capture in captures {
        matches.push(capture.get(1).unwrap().as_str());
    }

    matches
}

fn convert_from_parsed_folder_to_project(
    parsed: &ParsedFolder,
    folder_name: String,
    path: &str,
    files: &Option<BTreeMap<String, String>>,
) -> Result<Folder, String> {
    let childs = parsed
        .childs
        .iter()
        .map(|(child_name, child)| -> Result<FolderChild, String> {
            match child {
                ParsedStructureChild::File(file_content) => {
                let child_string = child_name.to_string();

                let (basename, extension) = child_string
                    .rsplit_once('.')
                    .map(|(basename, extension)| {
                        (basename.to_string(), extension.to_string())
                    })
                    .ok_or_else(|| {
                        format!(
                            "Invalid file name '{}' in test project structure",
                            child_string
                        )
                    })?;

                let content_to_use = if file_content.starts_with("use:") {
                    let file_name = file_content.replace("use:", "");

                    files
                        .as_ref()
                        .and_then(|files| files.get(&file_name))
                        .cloned()
                        .ok_or_else(|| {
                            format!(
                                "Missing file template '{}' in test project structure",
                                file_name
                            )
                        })?
                } else {
                    file_content.to_owned()
                };

                Ok(FolderChild::FileChild(File {
                    basename,
                    name_with_ext: child_string.clone(),
                    content: Some(content_to_use),
                    extension: Some(extension),
                    relative_path: format!("{}/{}", path, child_string),
                }))
                }
                ParsedStructureChild::Folder(folder) => {
                    Ok(FolderChild::Folder(convert_from_parsed_folder_to_project(
                    folder,
                    child_name.to_owned(),
                    format!("{}/{}", path, normalize_folder_config_name(child_name))
                        .as_str(),
                    files,
                )?))
                }
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Folder {
        name: normalize_folder_config_name(&folder_name),
        children: childs,
    })
}

fn parse_project_yaml(project_yaml: String) -> Result<Project, String> {
    let parsed_project_yaml: ParsedProjectYaml =
        serde_norway::from_str(&project_yaml).map_err(|err| err.to_string())?;

    let structure = convert_from_parsed_folder_to_project(
        &parsed_project_yaml.structure,
        ".".to_string(),
        ".",
        &parsed_project_yaml.files,
    )?;

    Ok(Project {
        only: parsed_project_yaml.only.unwrap_or(false),
        structure,
        expected_errors: match parsed_project_yaml.expected_errors {
            ExpectedErrors::Single(true) => None,
            ExpectedErrors::Single(false) => None,
            ExpectedErrors::Multiple(errors) => Some(errors),
        },
    })
}

fn get_test_cases(dir: &PathBuf) -> Result<Vec<TestCase>, String> {
    if !dir.exists() {
        return Err(format!(
            "Test cases folder '{}' does not exist",
            dir.display()
        ));
    }

    let mut test_cases = Vec::new();

    for entry in std::fs::read_dir(dir).map_err(|err| {
        format!(
            "Error reading test cases folder '{}': {}",
            dir.display(),
            err
        )
    })? {
        let entry = entry.map_err(|err| {
            format!(
                "Error reading an entry in test cases folder '{}': {}",
                dir.display(),
                err
            )
        })?;
        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        let file_name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .ok_or_else(|| {
                format!("Error getting file name for test case '{}'", path.display())
            })?;

        let file_content = std::fs::read_to_string(&path).map_err(|err| {
            format!("Error reading test case '{}': {}", path.display(), err)
        })?;

        test_cases.push(TestCase {
            file_name,
            file_content,
        });
    }

    Ok(test_cases)
}

fn do_vecs_match<T: Eq + Hash>(a: &[T], b: &[T]) -> bool {
    let a_set: HashSet<&T> = a.iter().collect();

    let b_set: HashSet<&T> = b.iter().collect();

    a_set == b_set
}

fn sort_vector(vector: &[String]) -> Vec<String> {
    let mut new_vec = vector.to_owned();

    new_vec.sort();

    new_vec
}

#[cfg(test)]
mod tests {
    use super::*;

    use insta::assert_debug_snapshot;

    fn strip_ansi_codes(text: &str) -> String {
        let ansi_regex = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        ansi_regex.replace_all(text, "").to_string()
    }

    #[test]
    fn test_config_success() {
        let test_summary = test_config(
            &PathBuf::from("./src/fixtures/cli_test_cases/test_cases_success"),
            &PathBuf::from("./src/fixtures/cli_test_cases/config.yaml"),
            false,
        );

        let stripped_result = match &test_summary {
            Ok(s) => Ok(strip_ansi_codes(s)),
            Err(s) => Err(strip_ansi_codes(s)),
        };

        assert_debug_snapshot!(stripped_result,
            @r###"
        Ok(
            "\n🟩 Running 1 test cases\n\n",
        )
        "###
        )
    }

    #[test]
    fn test_config_failure() {
        let test_summary = test_config(
            &PathBuf::from("./src/fixtures/cli_test_cases/test_cases_failure"),
            &PathBuf::from("./src/fixtures/cli_test_cases/config.yaml"),
            false,
        );

        let stripped_result = match &test_summary {
            Ok(s) => Ok(strip_ansi_codes(s)),
            Err(s) => Err(strip_ansi_codes(s)),
        };

        assert_debug_snapshot!(stripped_result,
            @r###"
        Err(
            "\n\n❌ Test case 'test.md' - project 3: Expected Ok but got errors: Problems {\n    errors: [\n        \"File ./stores/test_examples.ts:\\n • should be named in camelCase\",\n    ],\n    warnings: [],\n}\n\n\n🟩 Running 1 test cases\n\n",
        )
        "###
        )
    }

    #[test]
    fn test_config_failure_2() {
        let test_summary = test_config(
            &PathBuf::from("./src/fixtures/cli_test_cases/test_cases_failure_2"),
            &PathBuf::from("./src/fixtures/cli_test_cases/config.yaml"),
            false,
        );

        let stripped_result = match &test_summary {
            Ok(s) => Ok(strip_ansi_codes(s)),
            Err(s) => Err(strip_ansi_codes(s)),
        };

        assert_debug_snapshot!(stripped_result,
            @r###"
        Err(
            "\n\n❌ Test case 'test.md' - project 1: Expected errors but got Ok\n\n\n🟩 Running 1 test cases\n\n",
        )
        "###
        )
    }

    #[test]
    fn parse_project_with_files_templates() {
        let project_yaml = r###"
        only: true

        files:
            test: |
                import { test } from 'test';

                test();

        structure:
            /src:
                test.ts: use:test
        expected_errors: false
        "###;

        let parsed_project = parse_project_yaml(project_yaml.to_string()).unwrap();

        assert_debug_snapshot!(parsed_project,
            @r###"
        Project {
            only: true,
            structure: Folder {
                name: ".",
                children: [
                    Folder(
                        Folder {
                            name: "src",
                            children: [
                                FileChild(
                                    File {
                                        basename: "test",
                                        name_with_ext: "test.ts",
                                        content: Some(
                                            "import { test } from 'test';\n\ntest();\n",
                                        ),
                                        extension: Some(
                                            "ts",
                                        ),
                                        relative_path: "./src/test.ts",
                                    },
                                ),
                            ],
                        },
                    ),
                ],
            },
            expected_errors: None,
        }
        "###
        )
    }
}
