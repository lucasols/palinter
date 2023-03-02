use crate::{
    expect_checks::*,
    internal_config::{AnyOr, Config, FileConditions, FileExpect},
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

pub fn check_folder_childs(config: &Config, folder: Folder) -> Result<(), String> {
    for child in folder.childs {
        match child {
            Child::FileChild(file) => {
                for rule in &config.global_files_rules {
                    let file_matches = file_matches_condition(&file, &rule.conditions);

                    if file_matches {
                        return file_pass_expected(&file, &rule.expect);
                    }
                }
            }
            Child::Folder(sub_folder) => {
                check_folder_childs(&config, sub_folder)?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{internal_config::get_config, parse_config_file::parse_config_string};

    fn config_from_string(config_string: String) -> Config {
        let parsed_config = parse_config_string(config_string);

        get_config(&parsed_config)
    }

    #[test]
    fn global_config_file_name_case_rule_kebab_case() {
        let config = config_from_string(
            r#"
            {
                "global_rules": [
                    {
                        "if_file": { "has_extension": "svg" },
                        "expect": {
                            "name_case_is": "kebab-case"
                        },
                        "error_msg": "Svg files should be named in kebab-case"
                    }
                ]
            }
            "#
            .to_string(),
        );

        let result = check_folder_childs(
            &config,
            Folder {
                name: String::from("."),
                childs: vec![Child::FileChild(File {
                    name: String::from("icon-1"),
                    content: String::from("test"),
                    extension: String::from("svg"),
                })],
            },
        );

        if let Err(error) = result {
            panic!("{}", error);
        }

        let error_result = check_folder_childs(
            &config,
            Folder {
                name: String::from("."),
                childs: vec![Child::FileChild(File {
                    name: String::from("icon_1"),
                    content: String::from("test"),
                    extension: String::from("svg"),
                })],
            },
        );

        assert_eq!(
            error_result,
            Err(String::from("File 'icon_1' should be named in kebab-case"))
        );
    }

    #[test]
    fn file_name_case_rules() {
        let config = config_from_string(
            r#"
            {
                "//camelCase": {
                    "rules": [
                        {
                            "if_file": "any",
                            "expect": {
                                "name_case_is": "kebab-case"
                            }
                        }
                    ]
                },
                "//snake_case": {
                    "rules": [
                        {
                            "if_file": "any",
                            "expect": {
                                "name_case_is": "snake_case"
                            }
                        }
                    ]
                },
                "//PascalCase": {
                    "rules": [
                        {
                            "if_file": "any",
                            "expect": {
                                "name_case_is": "PascalCase"
                            }
                        }
                    ]
                },
            }
            "#
            .to_string(),
        );

        let result = check_folder_childs(
            &config,
            Folder {
                name: String::from("."),
                childs: vec![
                    Child::Folder(Folder {
                        name: String::from("camelCase"),
                        childs: vec![Child::FileChild(File {
                            name: String::from("camelCase"),
                            content: String::from("test"),
                            extension: String::from("svg"),
                        })],
                    }),
                    Child::Folder(Folder {
                        name: String::from("snake_case"),
                        childs: vec![Child::FileChild(File {
                            name: String::from("snake_case"),
                            content: String::from("test"),
                            extension: String::from("svg"),
                        })],
                    }),
                    Child::Folder(Folder {
                        name: String::from("PascalCase"),
                        childs: vec![Child::FileChild(File {
                            name: String::from("PascalCase"),
                            content: String::from("test"),
                            extension: String::from("svg"),
                        })],
                    }),
                ],
            },
        );

        if let Err(error) = result {
            panic!("{}", error);
        }

        let error_result = check_folder_childs(
            &config,
            Folder {
                name: String::from("."),
                childs: vec![Child::FileChild(File {
                    name: String::from("icon_1"),
                    content: String::from("test"),
                    extension: String::from("svg"),
                })],
            },
        );

        assert_eq!(
            error_result,
            Err(String::from("File 'icon_1' should be named in kebab-case"))
        );
    }
}
