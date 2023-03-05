use regex::Regex;
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;

use super::*;

use crate::{
    internal_config::get_config,
    parse_config_file::{parse_config, parse_config_string, ParseFrom},
};

fn config_from_string(config_string: String, parse_from: ParseFrom) -> Result<Config, String> {
    let parsed_config = parse_config_string(config_string, parse_from)?;

    get_config(&parsed_config)
}

#[derive(Debug)]
struct Project {
    structure: Folder,
    expected_errors: Option<Vec<String>>,
}

#[derive(Debug)]
struct TestCase {
    config: Config,
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

                let dot_parts = child_string.split('.');

                let extension = dot_parts.clone().last().unwrap().to_string();

                let file_name = dot_parts.clone().next().unwrap().to_string();

                Child::FileChild(File {
                    name: file_name,
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
        name: folder_name,
        childs,
    }
}

fn parse_project_yaml(project_yaml: String) -> Project {
    let parsed_project_yaml: ParsedProjectYaml = serde_yaml::from_str(&project_yaml).unwrap();

    let structure =
        convert_from_parsed_folder_to_project(&parsed_project_yaml.structure, ".".to_string());

    Project {
        structure,
        expected_errors: match parsed_project_yaml.expected_errors {
            ExpectedErrors::Single(true) => None,
            ExpectedErrors::Single(false) => None,
            ExpectedErrors::Multiple(errors) => Some(errors),
        },
    }
}

fn extract_config_and_projects_from_test_case(
    test_case_content: String,
) -> Result<TestCase, String> {
    let content_parts = test_case_content.split("# Projects");

    let config_part = content_parts.clone().next().unwrap().to_string();
    let projects_part = content_parts.clone().last().unwrap().to_string();

    let config_regex = Regex::new(r"```(json|yaml)\n([\S\s]+?)\n```").unwrap();

    let config_captures = config_regex.captures(&config_part).unwrap();

    let config_format = config_captures.get(1).unwrap().as_str();

    let config = config_captures.get(2).unwrap().as_str().to_string();

    let config = config_from_string(
        config,
        match config_format {
            "json" => ParseFrom::Json,
            "yaml" => ParseFrom::Yaml,
            _ => return Err("Invalid config format".to_string()),
        },
    )?;

    let projects_regex = Regex::new(r"\n```yaml\n([\S\s]+?)\n```").unwrap();

    let projects_captures = projects_regex.captures_iter(&projects_part);

    let projects: Vec<Project> = projects_captures
        .into_iter()
        .map(|project_capture| project_capture.get(1))
        .map(|project_capture| {
            let project_yaml = project_capture.unwrap().as_str().to_string();

            parse_project_yaml(project_yaml)
        })
        .collect();

    if projects.is_empty() {
        return Err("No projects found in test case".to_string());
    }

    Ok(TestCase { config, projects })
}

#[test]
fn test_cases() {
    let files_content = std::fs::read_dir("./src/test_cases")
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
            let file_content = std::fs::read_to_string(path).unwrap();

            (file_name, file_content)
        })
        .collect::<Vec<(String, String)>>();

    let is_dev = std::env::var("DEVTEST").is_ok();

    let mut ignored_test_cases = 0;

    let only_files_content_to_test: Vec<(String, String)> = files_content
        .iter()
        .filter(|(_, content)| content.starts_with("only"))
        .cloned()
        .collect();

    let files_content_to_test = if only_files_content_to_test.is_empty() {
        files_content
            .iter()
            .filter(|(_, content)| {
                if content.starts_with("ignore") {
                    ignored_test_cases += 1;

                    return false;
                }

                true
            })
            .cloned()
            .collect()
    } else {
        if !is_dev {
            panic!("Only test cases are not allowed in production, use 'DEVTEST=1' to run them")
        }

        println!(
            "\n🟧 Running only test cases with 'only' prefix ({} test cases)\n",
            only_files_content_to_test.len()
        );

        only_files_content_to_test
    };

    if ignored_test_cases > 0 {
        println!("\n🟧 Ignored {} test cases\n", ignored_test_cases);
    }

    for (file_name, file_content) in files_content_to_test {
        let test_case = extract_config_and_projects_from_test_case(file_content);

        match test_case {
            Err(error) => {
                panic!("\n\n❌ Test case '{}': {}\n\n", file_name, error);
            }
            Ok(test_case) => {
                for (i, project) in test_case.projects.iter().enumerate() {
                    let result = check_root_folder(&test_case.config, &project.structure);

                    let test_case =
                        format!("\n\n❌ Test case '{}' - project {}:", file_name, i + 1);

                    match &project.expected_errors {
                        Some(expected_errors) => {
                            if let Err(errors) = result {
                                assert_eq!(errors, *expected_errors, "{}\n\n", test_case);
                            } else {
                                panic!("{} Expected errors but got Ok\n\n", test_case);
                            }
                        }
                        None => {
                            if let Err(errors) = result {
                                panic!(
                                    "{} Expected Ok but got errors: {:?}\n\n",
                                    test_case, errors
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
