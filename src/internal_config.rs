use crate::parse_config_file::{ParsedAnyOr, ParsedConfig, ParsedRule, SingleOrMultiple};

pub enum AnyOr<T> {
    Any,
    Or(T),
}

pub enum NameCase {
    CamelCase,
    SnakeCase,
    KebabCase,
    PascalCase,
    ConstantCase,
}

pub struct FileExpect {
    pub name_case_is: Option<NameCase>,
}

pub struct FileConditions {
    pub has_extension: Option<Vec<String>>,
    // has_name: Option<Vec<String>>,
    // does_not_have_name: Option<Vec<String>>,
}

pub struct FileRule {
    pub conditions: AnyOr<FileConditions>,
    pub expect: AnyOr<Vec<FileExpect>>,
    // TODO: id: Option<String>,
}

pub struct Config {
    pub global_files_rules: Vec<FileRule>,
}

fn normalize_single_or_multiple<T: Clone>(single_or_multiple: &SingleOrMultiple<T>) -> Vec<T> {
    match single_or_multiple {
        SingleOrMultiple::Single(single) => vec![single.clone()],
        SingleOrMultiple::Multiple(multiple) => multiple.to_vec(),
    }
}

pub fn normalize_single_or_multiple_some<T: Clone>(
    single_or_multiple_option: &Option<SingleOrMultiple<T>>,
) -> Option<Vec<T>> {
    match single_or_multiple_option {
        Some(single_or_multiple) => Some(normalize_single_or_multiple(single_or_multiple)),
        None => None,
    }
}

fn check_any(any: &String) {
    if any != "any" {
        panic!("Invalid any: {}", any);
    }
}

fn normalize_name_case(name_case: &String) -> NameCase {
    match name_case.as_str() {
        "camelCase" => NameCase::CamelCase,
        "snake_case" => NameCase::SnakeCase,
        "kebab-case" => NameCase::KebabCase,
        "PascalCase" => NameCase::PascalCase,
        "CONSTANT_CASE" => NameCase::ConstantCase,
        _ => panic!("Invalid name case: {}", name_case),
    }
}

pub fn get_config(parsed_config: &ParsedConfig) -> Config {
    let mut global_files_rules: Vec<FileRule> = vec![];

    if let Some(parsed_global_rules) = &parsed_config.global_rules {
        for rule in parsed_global_rules {
            match rule {
                ParsedRule::File {
                    conditions, expect, ..
                } => {
                    let conditions = match conditions {
                        ParsedAnyOr::Any(any) => {
                            check_any(&any);
                            AnyOr::Any
                        }
                        ParsedAnyOr::Conditions(conditions) => AnyOr::Or(FileConditions {
                            has_extension: normalize_single_or_multiple_some(
                                &conditions.has_extension,
                            ),
                        }),
                    };

                    let new_expect = match expect {
                        ParsedAnyOr::Any(any) => {
                            check_any(&any);
                            AnyOr::Any
                        }
                        ParsedAnyOr::Conditions(expect_conditions) => {
                            let expects = normalize_single_or_multiple(&expect_conditions)
                                .iter()
                                .map(|parsed_expect| FileExpect {
                                    name_case_is: match &parsed_expect.name_case_is {
                                        Some(name_case_is) => {
                                            Some(normalize_name_case(name_case_is))
                                        }
                                        None => None,
                                    },
                                })
                                .collect();

                            AnyOr::Or(expects)
                        }
                    };

                    global_files_rules.push(FileRule {
                        conditions,
                        expect: new_expect,
                    });
                }
                _ => {}
            }
        }
    }

    Config { global_files_rules }
}
