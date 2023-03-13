use colored::Colorize;
use regex::Regex;
use serde::Deserialize;
use std::{collections::BTreeMap, hash::Hash};

use super::*;

use crate::{
    internal_config::get_config,
    parse_config_file::{parse_config_string, ParseFrom},
};

fn config_from_string(config_string: &String, parse_from: ParseFrom) -> Result<Config, String> {
    let parsed_config = parse_config_string(config_string, parse_from)?;

    get_config(&parsed_config)
}

#[derive(Debug)]
struct Project {
    only: bool,
    structure: Folder,
    expected_errors: Option<Vec<String>>,
}

#[derive(Debug)]
struct ProjectConfig {
    expect_config_error: Option<String>,
    config: Result<Config, String>,
}

#[derive(Debug)]
struct TestCase {
    configs: Vec<ProjectConfig>,
    projects: Vec<Project>,
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

fn convert_from_parsed_folder_to_project(parsed: &ParsedFolder, folder_name: String) -> Folder {
    let childs = parsed
        .childs
        .iter()
        .map(|(child_name, child)| match child {
            ParsedStructureChild::File(file_content) => {
                let child_string = child_name.to_string();

                let (basename, extension, sub_ext) = {
                    let parts = child_string.split('.').collect::<Vec<&str>>();

                    let basename = parts[0..parts.len() - 1].join(".");

                    let sub_extension = if parts.len() <= 2 {
                        None
                    } else {
                        parts.get(parts.len() - 2).map(|s| s.to_string())
                    };

                    let extension = parts.last().unwrap().to_string();

                    (basename, extension, sub_extension)
                };

                Child::FileChild(File {
                    basename,
                    name_with_ext: child_string,
                    sub_ext,
                    content: file_content.to_owned(),
                    extension,
                })
            }
            ParsedStructureChild::Folder(folder) => Child::Folder(
                convert_from_parsed_folder_to_project(folder, child_name.to_owned()),
            ),
        })
        .collect();

    Folder {
        name: normalize_folder_config_name(&folder_name),
        childs,
    }
}

fn parse_project_yaml(project_yaml: String) -> Result<Project, serde_yaml::Error> {
    let parsed_project_yaml: ParsedProjectYaml = serde_yaml::from_str(&project_yaml)?;

    let structure =
        convert_from_parsed_folder_to_project(&parsed_project_yaml.structure, ".".to_string());

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

fn extract_config_and_projects_from_test_case(
    test_case_content: String,
) -> Result<TestCase, String> {
    let (config_part, projects_part) = if test_case_content.contains("# Projects") {
        let mut content_parts = test_case_content.split("# Projects");

        let config_part = content_parts.next().unwrap().to_string();
        let projects_part = content_parts.next().unwrap().to_string();

        if content_parts.next().is_some() {
            return Err("Multiple # Projects found".to_string());
        }

        (config_part, projects_part)
    } else {
        (test_case_content, "".to_string())
    };

    let config_regex = Regex::new(r"```(json|yaml)\n([\S\s]+?)\n```").unwrap();

    let config_captures = config_regex.captures_iter(&config_part);

    let mut configs: Vec<ProjectConfig> = Vec::new();

    for config_capture in config_captures {
        let config_format = config_capture.get(1).unwrap().as_str();

        let config_string = config_capture.get(2).unwrap().as_str().to_string();

        let config_fist_line = config_string.lines().next().unwrap().to_string();

        let expect_config_error = if config_fist_line.starts_with("# expect_error: ") {
            Some(config_fist_line.replace("# expect_error: ", ""))
        } else {
            None
        };

        let config = config_from_string(
            &config_string,
            match config_format {
                "json" => ParseFrom::Json,
                "yaml" => ParseFrom::Yaml,
                _ => return Err("Invalid config format".to_string()),
            },
        );

        configs.push(ProjectConfig {
            expect_config_error,
            config,
        });
    }

    let projects_regex = Regex::new(r"\n```yaml\n([\S\s]+?)\n```").unwrap();

    let projects_captures = projects_regex.captures_iter(&projects_part);

    let mut projects: Vec<Project> = Vec::new();

    for (i, project_capture) in projects_captures.into_iter().enumerate() {
        let project_yaml = project_capture.get(1).unwrap().as_str().to_string();

        let project = parse_project_yaml(project_yaml);

        match project {
            Ok(project) => projects.push(project),
            Err(error) => return Err(format!("Error parsing project {} yaml: {}", i + 1, error)),
        }
    }

    if configs.is_empty() {
        return Err("No config found".to_string());
    }

    if configs.len() > 1 && !projects.is_empty() {
        return Err("Multiple configs and projects found".to_string());
    }

    Ok(TestCase { configs, projects })
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

#[test]
fn test_cases() {
    let files_content = get_test_cases("./src/test_cases");

    let is_dev = std::env::var("DEVTEST").is_ok();

    let mut test_sumary: String;

    let mut ignored_test_cases = 0;

    let only_files_content_to_test: Vec<(String, String)> = files_content
        .iter()
        .filter(|(_, content)| content.starts_with("only"))
        .cloned()
        .collect();

    let files_content_to_test = if only_files_content_to_test.is_empty() {
        let to_test: Vec<(String, String)> = files_content
            .iter()
            .filter(|(_, content)| {
                if content.starts_with("ignore") {
                    ignored_test_cases += 1;

                    return false;
                }

                true
            })
            .cloned()
            .collect();

        test_sumary = format!("\n🟩 Running {} test cases\n", to_test.len());

        to_test
    } else {
        if !is_dev {
            panic!("Only test cases are not allowed in production, use 'DEVTEST=1' to run them")
        }

        test_sumary = format!(
            "\n🟧 Running only test cases with 'only' prefix -> {}\n",
            only_files_content_to_test
                .clone()
                .into_iter()
                .map(|(name, _)| name)
                .collect::<Vec<String>>()
                .join(", ")
        );

        only_files_content_to_test
    };

    if ignored_test_cases > 0 {
        test_sumary = format!(
            "{}\n🟧 Ignored {} test cases\n",
            test_sumary, ignored_test_cases
        );
    }

    let mut test_errors: Vec<String> = vec![];

    for (file_name, file_content) in files_content_to_test {
        let test_case = extract_config_and_projects_from_test_case(file_content);

        match test_case {
            Err(error) => {
                test_errors.push(format!(
                    "\n\n❌ Test case '{}': {}",
                    file_name.blue(),
                    error
                ));
            }
            Ok(test_case) => {
                for (i, project_config) in test_case.configs.iter().enumerate() {
                    match &project_config.config {
                        Ok(config) => {
                            if project_config.expect_config_error.is_some() {
                                println!("{:#?}", config);

                                test_errors.push(format!(
                                    "❌ Test case '{}': Config {}: Expected config error but got Ok",
                                    file_name.blue(),
                                    i + 1
                                ));
                            }

                            let some_project_has_only =
                                test_case.projects.iter().any(|project| project.only);

                            if some_project_has_only {
                                if !is_dev {
                                    panic!("Only test cases are not allowed in production, use 'DEVTEST=1' to run them");
                                } else {
                                    test_sumary = format!(
                                        "{}\n🟧 Running projects with only flag!\n",
                                        test_sumary
                                    );
                                }
                            }

                            for (i, project) in test_case
                                .projects
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
                                let result = check_root_folder(config, &project.structure);

                                let test_case = format!(
                                    "❌ Test case '{}' - project {}:",
                                    file_name.blue(),
                                    i + 1
                                );

                                match &project.expected_errors {
                                    Some(expected_errors) => {
                                        if let Err(errors) = result {
                                            if !do_vecs_match(&errors, expected_errors) {
                                                test_errors.push(format!(
                                                    "{}\n\
                                                    Expected errors: {:#?}\n\
                                                    But got:         {:#?}",
                                                    test_case,
                                                    sort_vector(expected_errors),
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
                        Err(error) => {
                            if let Some(expect_config_error) = &project_config.expect_config_error {
                                if error != expect_config_error {
                                    test_errors.push(format!(
                                        "❌ Test case '{}': Config {}:\n\
                                            Expected config error: '{}'\n\
                                            But got:               '{}'",
                                        file_name.blue(),
                                        i + 1,
                                        expect_config_error,
                                        error
                                    ));
                                }
                            } else {
                                test_errors.push(format!(
                                    "❌ Test case '{}': Config {}: Expected Ok but got error: {}",
                                    file_name.blue(),
                                    i + 1,
                                    error
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    if !test_errors.is_empty() {
        panic!("\n\n{}\n\n{}\n", test_errors.join("\n\n"), test_sumary);
    } else {
        println!("{}\n", test_sumary);
    }
}

fn get_test_cases(dir: &str) -> Vec<(String, String)> {
    std::fs::read_dir(dir)
        .unwrap()
        .flat_map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                get_test_cases(path.to_str().unwrap())
            } else {
                let file_name = path.file_name().unwrap().to_str().unwrap().to_string();

                if file_name == "example.md" {
                    return vec![];
                }

                let file_content = std::fs::read_to_string(path).unwrap();

                vec![(file_name, file_content)]
            }
        })
        .collect::<Vec<(String, String)>>()
}
