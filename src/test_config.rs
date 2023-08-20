use colored::Colorize;
use regex::Regex;
use std::{
    collections::{BTreeMap, HashSet},
    hash::Hash,
    path::{Path, PathBuf},
};

use crate::{
    analyze_ts_deps::{_setup_test, load_used_project_files_deps_info_from_cfg},
    check_folders::{check_root_folder, normalize_folder_config_name},
    internal_config::get_config,
    load_folder_structure::{File, Folder, FolderChild},
    parse_config_file,
    test_utils::TEST_MUTEX,
};
use serde::Deserialize;

#[derive(Debug, Clone)]
struct TestCase {
    file_name: String,
    file_content: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum ParsedStructureChild {
    Folder(ParsedFolder),
    File(String),
}

#[derive(Deserialize, Debug)]
struct ParsedFolder {
    #[serde(flatten)]
    childs: BTreeMap<String, ParsedStructureChild>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum ExpectedErrors {
    Single(bool),
    Multiple(Vec<String>),
}

#[derive(Deserialize, Debug)]
struct ParsedProjectYaml {
    only: Option<bool>,
    structure: ParsedFolder,
    expected_errors: ExpectedErrors,
}

struct Project {
    only: bool,
    structure: Folder,
    expected_errors: Option<Vec<String>>,
}

pub fn test_config(
    test_cases_dir: &PathBuf,
    config_file: &PathBuf,
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

                    let result = check_root_folder(&config, &project.structure);

                    colored::control::unset_override();

                    let test_case = format!(
                        "❌ Test case '{}' - project {}:",
                        file_name.blue(),
                        i + 1
                    );

                    match &project.expected_errors {
                        Some(expected_errors) => {
                            if let Err(errors) = result {
                                let collected = &expected_errors
                                    .iter()
                                    .map(|err| err.trim().to_string())
                                    .collect::<Vec<String>>();

                                if !do_vecs_match(&errors, collected) {
                                    test_errors.push(format!(
                                        "{}\n\
                                                    Expected errors: {:#?}\n\
                                                    But got:         {:#?}",
                                        test_case,
                                        sort_vector(collected),
                                        sort_vector(&errors)
                                    ));
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

fn extract_projects_from_file_content(
    test_case_content: String,
) -> Result<Vec<Project>, String> {
    let mut projects: Vec<Project> = Vec::new();

    let projects_regex = Regex::new(r"```yaml\n([\S\s]+?)\n```").unwrap();

    let projects_captures = projects_regex.captures_iter(&test_case_content);

    for (i, project_capture) in projects_captures.into_iter().enumerate() {
        let project_yaml = project_capture.get(1).unwrap().as_str().to_string();

        let project = parse_project_yaml(project_yaml);

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

fn convert_from_parsed_folder_to_project(
    parsed: &ParsedFolder,
    folder_name: String,
    path: &str,
) -> Folder {
    let childs = parsed
        .childs
        .iter()
        .map(|(child_name, child)| match child {
            ParsedStructureChild::File(file_content) => {
                let child_string = child_name.to_string();

                let (basename, extension) = {
                    let parts = child_string.split('.').collect::<Vec<&str>>();

                    let basename = parts[0..parts.len() - 1].join(".");

                    let extension = parts.last().unwrap().to_string();

                    (basename, extension)
                };

                FolderChild::FileChild(File {
                    basename,
                    name_with_ext: child_string.clone(),
                    content: Some(file_content.to_owned()),
                    extension: Some(extension),
                    relative_path: format!("{}/{}", path, child_string),
                })
            }
            ParsedStructureChild::Folder(folder) => {
                FolderChild::Folder(convert_from_parsed_folder_to_project(
                    folder,
                    child_name.to_owned(),
                    format!("{}/{}", path, normalize_folder_config_name(child_name))
                        .as_str(),
                ))
            }
        })
        .collect();

    Folder {
        name: normalize_folder_config_name(&folder_name),
        childs,
    }
}

fn parse_project_yaml(project_yaml: String) -> Result<Project, serde_yaml::Error> {
    let parsed_project_yaml: ParsedProjectYaml =
        serde_yaml::from_str(&project_yaml)?;

    let structure = convert_from_parsed_folder_to_project(
        &parsed_project_yaml.structure,
        ".".to_string(),
        ".",
    );

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

    Ok(std::fs::read_dir(dir)
        .unwrap()
        .flat_map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                get_test_cases(&path).unwrap()
            } else {
                let file_name =
                    path.file_name().unwrap().to_str().unwrap().to_string();

                let file_content = std::fs::read_to_string(path).unwrap();

                vec![TestCase {
                    file_name,
                    file_content,
                }]
            }
        })
        .collect::<Vec<TestCase>>())
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

    #[test]
    fn test_config_success() {
        let test_summary = test_config(
            &PathBuf::from("./src/fixtures/cli_test_cases/test_cases_success"),
            &PathBuf::from("./src/fixtures/cli_test_cases/config.yaml"),
        );

        assert_debug_snapshot!(test_summary,
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
        );

        assert_debug_snapshot!(test_summary,
            @r###"
        Err(
            "\n\n❌ Test case '\u{1b}[34mtest.md\u{1b}[0m' - project 3: Expected Ok but got errors: [\n    \"File ./stores/test_examples.ts:\\n • should be named in camelCase\",\n]\n\n\n🟩 Running 1 test cases\n\n",
        )
        "###
        )
    }

    #[test]
    fn test_config_failure_2() {
        let test_summary = test_config(
            &PathBuf::from("./src/fixtures/cli_test_cases/test_cases_failure_2"),
            &PathBuf::from("./src/fixtures/cli_test_cases/config.yaml"),
        );

        assert_debug_snapshot!(test_summary,
            @r###"
        Err(
            "\n\n❌ Test case '\u{1b}[34mtest.md\u{1b}[0m' - project 1: Expected errors but got Ok\n\n\n🟩 Running 1 test cases\n\n",
        )
        "###
        )
    }
}
