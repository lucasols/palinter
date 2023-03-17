use regex::Regex;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum ImportType {
    Named(Vec<String>),
    All,
    DefaultAndNamed(Vec<String>),
}

#[derive(Debug, PartialEq)]
pub struct Import {
    pub import_path: PathBuf,
    pub line: usize,
    pub values: ImportType,
}

pub fn extract_imports_from_file_content(file_content: &str) -> Vec<Import> {
    let lines = file_content.lines();
    let mut imports = Vec::new();
    let mut line_number = 0;
    let path_regex: Regex = Regex::new(r#"["'](.+)["']"#).unwrap();
    let named_values_regex = Regex::new(r#"\{(.+)\}"#).unwrap();
    let multiline_values_end_regex = Regex::new(r#"(.*)\}"#).unwrap();

    #[derive(Default, Clone, PartialEq)]
    struct MultilineResult {
        values: Vec<String>,
        start: usize,
    }

    let mut multiline_res: MultilineResult = MultilineResult::default();

    for line in lines {
        line_number += 1;

        if line.trim().starts_with("import") {
            multiline_res.start = line_number;

            if let Some((values_part, path_part)) = split_string_by(line, "from") {
                let mut add_import = |import: Import| {
                    imports.push(import);

                    multiline_res = MultilineResult::default();
                };

                let import_path = path_regex
                    .captures(&path_part)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str();

                if line.contains('*') {
                    add_import(Import {
                        import_path: PathBuf::from(import_path),
                        line: line_number,
                        values: ImportType::All,
                    });
                } else if line.contains('{') && line.contains('}') {
                    let captures = named_values_regex.captures(&values_part).unwrap();

                    let values = captures
                        .get(1)
                        .unwrap()
                        .as_str()
                        .split(',')
                        .map(|s| {
                            if s.contains(" as ") {
                                let split = s.split(" as ");

                                split.clone().next().unwrap().trim().to_string()
                            } else {
                                s.trim().to_string()
                            }
                        })
                        .collect::<Vec<String>>();

                    add_import(Import {
                        import_path: PathBuf::from(import_path),
                        line: line_number,
                        values: ImportType::Named(values),
                    });
                } else {
                    add_import(Import {
                        import_path: PathBuf::from(import_path),
                        line: line_number,
                        values: ImportType::Named(vec![]),
                    });
                }
            }
        } else if multiline_res != MultilineResult::default() {
            if Some((values_part, path_part)) = split_string_by(line, "from") {
                let import_path = path_regex
                    .captures(&path_part)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str();

                let values = multiline_values_end_regex
                    .captures(&values_part)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .split(',')
                    .map(|s| {
                        if s.contains(" as ") {
                            let split = s.split(" as ");

                            split.clone().next().unwrap().trim().to_string()
                        } else {
                            s.trim().to_string()
                        }
                    })
                    .collect::<Vec<String>>();

                imports.push(Import {
                    import_path: PathBuf::from(import_path),
                    line: multiline_res.start,
                    values: ImportType::Named(values),
                });

                multiline_res = MultilineResult::default();
            } else {

            }
        }
    }

    imports
}

fn split_string_by(string: &str, split_by: &str) -> Option<(String, String)> {
    let split = string.split(split_by).collect::<Vec<&str>>();

    if split.len() == 2 {
        Some((split[0].to_string(), split[1].to_string()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_imports() {
        let file_content = "const foo = 'bar';";
        let imports = extract_imports_from_file_content(file_content);
        assert_eq!(imports.len(), 0);
    }

    #[test]
    fn test_single_import() {
        let file_content = r#"import { Foo } from '@src/foo';"#;
        let imports = extract_imports_from_file_content(file_content);

        assert_eq!(
            vec![Import {
                import_path: PathBuf::from("@src/foo"),
                line: 1,
                values: ImportType::Named(vec!["Foo".to_string()]),
            }],
            imports
        );
    }

    #[test]
    fn test_multiple_imports() {
        let file_content = r#"
            import { Foo, Bar } from '@src/foo';
            import { Baz } from "@src/baz";
            import { Qux } from '@src/qux';
        "#;
        let imports = extract_imports_from_file_content(file_content);

        assert_eq!(
            vec![
                Import {
                    import_path: PathBuf::from("@src/foo"),
                    line: 2,
                    values: ImportType::Named(vec!["Foo".to_string(), "Bar".to_string()])
                },
                Import {
                    import_path: PathBuf::from("@src/baz"),
                    line: 3,
                    values: ImportType::Named(vec!["Baz".to_string()]),
                },
                Import {
                    import_path: PathBuf::from("@src/qux"),
                    line: 4,
                    values: ImportType::Named(vec!["Qux".to_string()]),
                },
            ],
            imports
        );
    }

    #[test]
    fn test_import_all() {
        let file_content = r#"import * as Foo from '@src/foo';"#;
        let imports = extract_imports_from_file_content(file_content);

        assert_eq!(
            vec![Import {
                import_path: PathBuf::from("@src/foo"),
                line: 1,
                values: ImportType::All,
            }],
            imports
        );
    }

    #[test]
    fn test_import_with_alias() {
        let file_content = r#"import { Foo as Bar } from '@src/foo';"#;
        let imports = extract_imports_from_file_content(file_content);

        assert_eq!(
            vec![Import {
                import_path: PathBuf::from("@src/foo"),
                line: 1,
                values: ImportType::Named(vec!["Foo".to_string()]),
            }],
            imports
        );
    }

    #[test]
    fn test_import_default() {
        let file_content = r#"import Foo from '@src/foo';"#;
        let imports = extract_imports_from_file_content(file_content);

        assert_eq!(
            vec![Import {
                import_path: PathBuf::from("@src/foo"),
                line: 1,
                values: ImportType::DefaultAndNamed(vec![]),
            }],
            imports
        );
    }

    #[test]
    fn test_import_default_and_named() {
        let file_content = r#"import Foo, { Bar, Baz } from '@src/foo';"#;
        let imports = extract_imports_from_file_content(file_content);

        assert_eq!(
            vec![Import {
                import_path: PathBuf::from("@src/foo"),
                line: 1,
                values: ImportType::DefaultAndNamed(vec!["Bar".to_string(), "Baz".to_string()]),
            }],
            imports
        );
    }

    #[test]
    fn test_import_commented() {
        let file_content = r#"
            // import { Foo } from '@src/foo';
            import { Bar } from '@src/bar';
        "#;
        let imports = extract_imports_from_file_content(file_content);

        assert_eq!(
            vec![Import {
                import_path: PathBuf::from("@src/bar"),
                line: 3,
                values: ImportType::Named(vec!["Bar".to_string()]),
            }],
            imports
        );
    }

    #[test]
    fn test_import_multiline() {
        let file_content = r#"
            import {
                Foo,
                Bar,
            } from '@src/foo';
        "#;
        let imports = extract_imports_from_file_content(file_content);

        assert_eq!(
            vec![Import {
                import_path: PathBuf::from("@src/foo"),
                line: 2,
                values: ImportType::Named(vec!["Foo".to_string(), "Bar".to_string()]),
            }],
            imports
        );
    }
}
