use crate::parse_config_file::{AnyOr, Config, Rule, SingleOrMultiple};

#[derive(Debug)]
enum Child {
    File {
        name: String,
        content: String,
        extension: String,
    },
    Folder(Folder),
}

#[derive(Debug)]
pub struct Folder {
    name: String,
    childs: Vec<Child>,
}

pub fn check_files(config: &Config, folder: Folder) -> Result<(), String> {
    for child in folder.childs {
        match child {
            Child::File {
                name,
                content,
                extension,
            } => match &config.global_rules {
                Some(rules) => {
                    for rule in rules {
                        match rule {
                            Rule::File { conditions, .. } => {
                                let mut file_matches = false;

                                match conditions {
                                    AnyOr::Conditions(conditions) => {
                                        match &conditions.has_extension {
                                            Some(rule_extensions) => {
                                                let extensions: Vec<String> = match rule_extensions {
                                                    SingleOrMultiple::Single(extension) => {
                                                        vec![extension.clone()]
                                                    }
                                                    SingleOrMultiple::Multiple(extensions) => {
                                                        extensions.to_vec()
                                                    }
                                                };

                                                if extensions.contains(&extension) {
                                                    file_matches = true;
                                                }

                                                if file_matches {

                                                }
                                            }
                                            None => {}
                                        }
                                    }
                                    AnyOr::Any(any) => {
                                        println!("Any {}", any);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                None => {}
            },
            Child::Folder(sub_folder) => {
                check_files(&config, sub_folder)?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod global_rules {
        use crate::parse_config_file::parse_config_string;

        use super::*;

        #[test]
        fn file_name_case_rule_kebab_case() {
            let config = parse_config_string(
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

            let root_structure = Folder {
                name: String::from("."),
                childs: vec![Child::File {
                    name: String::from("icon-1"),
                    content: String::from("test"),
                    extension: String::from("svg"),
                }],
            };

            let result = check_files(&config, root_structure);

            assert!(result.is_ok());
        }
    }
}
