use lazy_static::lazy_static;
use regex::Regex;
use std::path::PathBuf;

use crate::utils::split_string_by;

#[derive(Debug, PartialEq)]
pub enum ImportType {
    Named(Vec<String>),
    All,
    Dynamic,
}

#[derive(Debug, PartialEq)]
pub struct Import {
    pub import_path: PathBuf,
    pub line: usize,
    pub values: ImportType,
}

const DEFAULT: &str = "default";

pub fn extract_imports_from_file_content(
    file_content: &str,
) -> Result<Vec<Import>, String> {
    let lines = file_content.lines();
    let mut imports = Vec::new();

    lazy_static! {
        static ref PATH_REGEX: Regex = Regex::new(r#"["'](.+)["']"#).unwrap();
        static ref NAMED_VALUES_RE: Regex = Regex::new(r#"\{(.+)\}"#).unwrap();
        static ref MULTILINE_VALUES_END_REGEX: Regex =
            Regex::new(r#"(.*)\}"#).unwrap();
        static ref IS_DEFAULT_IMPORT: Regex = Regex::new(r#"import\s+\w"#).unwrap();
    }

    #[derive(Default, Clone, PartialEq)]
    struct MultilineResult {
        values: Vec<String>,
        start: usize,
        has_default: bool,
        is_dynamic: bool,
    }

    let mut current_line = 0;

    let lines = lines.collect::<Vec<&str>>();

    while current_line < lines.len() {
        current_line += 1;

        let line = lines[current_line - 1];

        if line.trim().starts_with("import") {
            // the full import statement is on one line
            if let Some((values_part, path_part)) = split_string_by(line, "from") {
                let mut add_import = |import: Import| {
                    imports.push(import);
                };

                let import_path = PATH_REGEX
                    .captures(&path_part)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str();

                // import is a namespace import `* as foo`
                if line.contains('*') {
                    add_import(Import {
                        import_path: PathBuf::from(import_path),
                        line: current_line,
                        values: ImportType::All,
                    });

                // import has named imports `{ foo, bar }`
                } else if line.contains('{') && line.contains('}') {
                    let captures = NAMED_VALUES_RE.captures(&values_part).unwrap();

                    let mut values = captures
                        .get(1)
                        .unwrap()
                        .as_str()
                        .split(',')
                        .filter_map(filter_map_named_import_value)
                        .collect::<Vec<String>>();

                    if IS_DEFAULT_IMPORT.is_match(&values_part) {
                        values.push(DEFAULT.to_string());
                    }

                    add_import(Import {
                        import_path: PathBuf::from(import_path),
                        line: current_line,
                        values: ImportType::Named(values),
                    });

                // import is default import only
                } else {
                    add_import(Import {
                        import_path: PathBuf::from(import_path),
                        line: current_line,
                        values: ImportType::Named(vec![DEFAULT.to_string()]),
                    });
                }

            // multiline import
            } else {
                let multiline_start = current_line;
                let mut has_default = false;
                let mut multiline_values: Vec<String> = Vec::new();

                if IS_DEFAULT_IMPORT.is_match(line) {
                    has_default = true;
                }

                lazy_static! {
                    static ref MULTILINE_START_VALUES: Regex =
                        Regex::new(r#"\{\s+(.+)"#).unwrap();
                }

                if let Some(captures) = MULTILINE_START_VALUES.captures(line) {
                    let values = captures
                        .get(1)
                        .unwrap()
                        .as_str()
                        .split(',')
                        .filter_map(filter_map_named_import_value)
                        .collect::<Vec<String>>();

                    multiline_values.extend(values);
                }

                // parse the next lines until the end of the multiline import

                let mut is_in_multiline_import = true;

                while is_in_multiline_import {
                    current_line += 1;

                    let line = lines[current_line - 1];

                    // is the end of a multiline import
                    if let Some((values_part, path_part)) =
                        split_string_by(line, "from")
                    {
                        let import_path = PATH_REGEX
                            .captures(&path_part)
                            .unwrap()
                            .get(1)
                            .unwrap()
                            .as_str();

                        let values = MULTILINE_VALUES_END_REGEX
                            .captures(&values_part)
                            .unwrap()
                            .get(1)
                            .unwrap()
                            .as_str()
                            .split(',')
                            .filter_map(filter_map_named_import_value)
                            .collect::<Vec<String>>();

                        multiline_values.extend(values);

                        if has_default {
                            multiline_values.push(DEFAULT.to_string());
                        }

                        imports.push(Import {
                            import_path: PathBuf::from(import_path),
                            line: multiline_start,
                            values: ImportType::Named(multiline_values.clone()),
                        });

                        is_in_multiline_import = false;

                    // is a line in the middle of a multiline import
                    } else {
                        let values = line
                            .split(',')
                            .filter_map(filter_map_named_import_value)
                            .collect::<Vec<String>>();

                        multiline_values.extend(values);
                    }
                }
            }
        } else if line.trim().starts_with("export") {
            // the full export statement is on one line
            if let Some((values_part, path_part)) = split_string_by(line, "from") {
                let import_path = PATH_REGEX
                    .captures(&path_part)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str();

                // export is a namespace export `* as foo`
                if line.contains('*') {
                    imports.push(Import {
                        import_path: PathBuf::from(import_path),
                        line: current_line,
                        values: ImportType::All,
                    });

                // export has named export `{ foo, bar }`
                } else if line.contains('{') && line.contains('}') {
                    let captures = NAMED_VALUES_RE.captures(&values_part).unwrap();

                    let mut values = captures
                        .get(1)
                        .unwrap()
                        .as_str()
                        .split(',')
                        .filter_map(filter_map_named_import_value)
                        .collect::<Vec<String>>();

                    if IS_DEFAULT_IMPORT.is_match(&values_part) {
                        values.push(DEFAULT.to_string());
                    }

                    imports.push(Import {
                        import_path: PathBuf::from(import_path),
                        line: current_line,
                        values: ImportType::Named(values),
                    });
                }

            // multiline reexport
            } else {
                let multiline_start = current_line;
                let mut multiline_values: Vec<String> = Vec::new();

                lazy_static! {
                    static ref MULTILINE_START_VALUES: Regex =
                        Regex::new(r#"\{\s+(.+)"#).unwrap();
                }

                if let Some(captures) = MULTILINE_START_VALUES.captures(line) {
                    let values = captures
                        .get(1)
                        .unwrap()
                        .as_str()
                        .split(',')
                        .filter_map(filter_map_named_import_value)
                        .collect::<Vec<String>>();

                    multiline_values.extend(values);
                }

                // parse the next lines until the end of the multiline export

                let mut is_in_multiline_export = true;

                while is_in_multiline_export {
                    current_line += 1;

                    let line = lines[current_line - 1];

                    lazy_static! {
                        static ref IS_SIMPLE_MULTILINE_EXPORT: Regex =
                            Regex::new(r#"\};?\s*$"#).unwrap();
                    }

                    if IS_SIMPLE_MULTILINE_EXPORT.is_match(line) {
                        is_in_multiline_export = false;
                        continue;
                    }

                    // is the end of a multiline export
                    if let Some((values_part, path_part)) =
                        split_string_by(line, "from")
                    {
                        let import_path = PATH_REGEX
                            .captures(&path_part)
                            .unwrap()
                            .get(1)
                            .unwrap()
                            .as_str();

                        let values = MULTILINE_VALUES_END_REGEX
                            .captures(&values_part)
                            .unwrap()
                            .get(1)
                            .unwrap()
                            .as_str()
                            .split(',')
                            .filter_map(filter_map_named_import_value)
                            .collect::<Vec<String>>();

                        multiline_values.extend(values);

                        imports.push(Import {
                            import_path: PathBuf::from(import_path),
                            line: multiline_start,
                            values: ImportType::Named(multiline_values.clone()),
                        });

                        is_in_multiline_export = false;

                    // is a line in the middle of a multiline export
                    } else {
                        let values = line
                            .split(',')
                            .filter_map(filter_map_named_import_value)
                            .collect::<Vec<String>>();

                        multiline_values.extend(values);
                    }
                }
            }
        } else {
            lazy_static! {
                static ref DYNAMIC_IMPORT: Regex =
                    Regex::new(r#"import\(['"](.+)['"]\)"#).unwrap();
                static ref IS_MULTILINE_DYNAMIC_IMPORT: Regex =
                    Regex::new(r#"import\(\s*$"#).unwrap();
                static ref PATH_AFTER_MULTILINE_DYNAMIC_IMPORT: Regex =
                    Regex::new(r#"^\s*['"](.+)['"]"#).unwrap();
            }

            if IS_MULTILINE_DYNAMIC_IMPORT.is_match(line) {
                let start_line = current_line;

                current_line += 1;

                let next_line = lines[current_line - 1];

                let import_path = PATH_AFTER_MULTILINE_DYNAMIC_IMPORT
                    .captures(next_line)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str();

                imports.push(Import {
                    import_path: PathBuf::from(import_path),
                    line: start_line,
                    values: ImportType::Dynamic,
                });
            } else {
                for capture in DYNAMIC_IMPORT.captures_iter(line) {
                    let import_path = capture.get(1).unwrap().as_str();

                    imports.push(Import {
                        import_path: PathBuf::from(import_path),
                        line: current_line,
                        values: ImportType::Dynamic,
                    });
                }
            }
        }
    }

    Ok(imports)
}

fn filter_map_named_import_value(captured_string: &str) -> Option<String> {
    let value = if captured_string.contains(" as ") {
        let split = captured_string.split(" as ");

        split.clone().next().unwrap().trim().to_string()
    } else {
        captured_string.trim().to_string()
    };

    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(test)]
    use pretty_assertions::assert_eq;

    #[test]
    fn no_imports() {
        let file_content = "const foo = 'bar';";
        let imports = extract_imports_from_file_content(file_content).unwrap();
        assert_eq!(imports.len(), 0);
    }

    #[test]
    fn single_import() {
        let file_content = r#"import { Foo } from '@src/foo';"#;
        let imports = extract_imports_from_file_content(file_content).unwrap();

        assert_eq!(
            imports,
            vec![Import {
                import_path: PathBuf::from("@src/foo"),
                line: 1,
                values: ImportType::Named(vec!["Foo".to_string()]),
            }],
        );
    }

    #[test]
    fn multiple_imports() {
        let file_content = r#"
            import { Foo, Bar } from '@src/foo';
            import { Baz } from "@src/baz";
            import { Qux } from '@src/qux';
        "#;
        let imports = extract_imports_from_file_content(file_content).unwrap();

        assert_eq!(
            imports,
            vec![
                Import {
                    import_path: PathBuf::from("@src/foo"),
                    line: 2,
                    values: ImportType::Named(vec![
                        "Foo".to_string(),
                        "Bar".to_string()
                    ])
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
        );
    }

    #[test]
    fn import_all() {
        let file_content = r#"import * as Foo from '@src/foo';"#;
        let imports = extract_imports_from_file_content(file_content).unwrap();

        assert_eq!(
            imports,
            vec![Import {
                import_path: PathBuf::from("@src/foo"),
                line: 1,
                values: ImportType::All,
            }],
        );
    }

    #[test]
    fn import_with_alias() {
        let file_content = r#"import { Foo as Bar } from '@src/foo';"#;
        let imports = extract_imports_from_file_content(file_content).unwrap();

        assert_eq!(
            imports,
            vec![Import {
                import_path: PathBuf::from("@src/foo"),
                line: 1,
                values: ImportType::Named(vec!["Foo".to_string()]),
            }],
        );
    }

    #[test]
    fn import_default() {
        let file_content = r#"import Foo from '@src/foo';"#;
        let imports = extract_imports_from_file_content(file_content).unwrap();

        assert_eq!(
            imports,
            vec![Import {
                import_path: PathBuf::from("@src/foo"),
                line: 1,
                values: ImportType::Named(vec![DEFAULT.to_string()]),
            }],
        );
    }

    #[test]
    fn import_default_and_named() {
        let file_content = r#"import Foo, { Bar, Baz } from '@src/foo';"#;
        let imports = extract_imports_from_file_content(file_content).unwrap();

        assert_eq!(
            imports,
            vec![Import {
                import_path: PathBuf::from("@src/foo"),
                line: 1,
                values: ImportType::Named(vec![
                    "Bar".to_string(),
                    "Baz".to_string(),
                    DEFAULT.to_string(),
                ]),
            }],
        );
    }

    #[test]
    fn import_commented() {
        let file_content = r#"
            // import { Foo } from '@src/foo';
            import { Bar } from '@src/bar';
        "#;
        let imports = extract_imports_from_file_content(file_content).unwrap();

        assert_eq!(
            imports,
            vec![Import {
                import_path: PathBuf::from("@src/bar"),
                line: 3,
                values: ImportType::Named(vec!["Bar".to_string()]),
            }],
        );
    }

    #[test]
    fn import_multiline() {
        let file_content = r#"
            import {
                Foo,
                Bar,
            } from '@src/foo';
        "#;
        let imports = extract_imports_from_file_content(file_content).unwrap();

        assert_eq!(
            imports,
            vec![Import {
                import_path: PathBuf::from("@src/foo"),
                line: 2,
                values: ImportType::Named(vec![
                    "Foo".to_string(),
                    "Bar".to_string()
                ]),
            }],
        );
    }

    #[test]
    fn import_multiline_with_alias() {
        let file_content = r#"
            import {
                Foo as Bar,
                Baz,
            } from '@src/foo';
        "#;
        let imports = extract_imports_from_file_content(file_content).unwrap();

        assert_eq!(
            imports,
            vec![Import {
                import_path: PathBuf::from("@src/foo"),
                line: 2,
                values: ImportType::Named(vec![
                    "Foo".to_string(),
                    "Baz".to_string()
                ]),
            }],
        );
    }

    #[test]
    fn import_multiline_with_default() {
        let file_content = r#"
            import Foo, {
                Bar,
                Baz,
            } from '@src/foo';
        "#;
        let imports = extract_imports_from_file_content(file_content).unwrap();

        assert_eq!(
            imports,
            vec![Import {
                import_path: PathBuf::from("@src/foo"),
                line: 2,
                values: ImportType::Named(vec![
                    "Bar".to_string(),
                    "Baz".to_string(),
                    DEFAULT.to_string(),
                ]),
            }],
        );
    }

    #[test]
    fn import_multiline_messed_up() {
        let file_content = r#"
            import Foo, {
                Bar,
                Baz, Test, Ok,
            What,
            } from '@src/foo';
            import { Qux } from '@src/qux';
        "#;
        let imports = extract_imports_from_file_content(file_content).unwrap();

        assert_eq!(
            imports,
            vec![
                Import {
                    import_path: PathBuf::from("@src/foo"),
                    line: 2,
                    values: ImportType::Named(vec![
                        "Bar".to_string(),
                        "Baz".to_string(),
                        "Test".to_string(),
                        "Ok".to_string(),
                        "What".to_string(),
                        DEFAULT.to_string(),
                    ]),
                },
                Import {
                    import_path: PathBuf::from("@src/qux"),
                    line: 7,
                    values: ImportType::Named(vec!["Qux".to_string()]),
                },
            ],
        );
    }

    #[test]
    fn import_multiline_messed_up_2() {
        let file_content = r#"
            import Foo, { Test2, What2,
                Bar,
                Baz, Test, Ok,
            What,
            } from '@src/foo';
        "#;
        let imports = extract_imports_from_file_content(file_content).unwrap();

        assert_eq!(
            imports,
            vec![Import {
                import_path: PathBuf::from("@src/foo"),
                line: 2,
                values: ImportType::Named(vec![
                    "Test2".to_string(),
                    "What2".to_string(),
                    "Bar".to_string(),
                    "Baz".to_string(),
                    "Test".to_string(),
                    "Ok".to_string(),
                    "What".to_string(),
                    DEFAULT.to_string(),
                ]),
            },],
        );
    }

    #[test]
    fn dynamic_import() {
        let file_content = r#"
            const test = import('@src/foo');
        "#;
        let imports = extract_imports_from_file_content(file_content).unwrap();

        assert_eq!(
            imports,
            vec![Import {
                import_path: PathBuf::from("@src/foo"),
                line: 2,
                values: ImportType::Dynamic,
            }],
        );
    }

    #[test]
    fn multiline_dynamic_import() {
        let file_content = r#"
            const test = import(
                '@src/foo'
            );
        "#;
        let imports = extract_imports_from_file_content(file_content).unwrap();

        assert_eq!(
            imports,
            vec![Import {
                import_path: PathBuf::from("@src/foo"),
                line: 2,
                values: ImportType::Dynamic,
            }],
        );
    }

    #[test]
    fn reexport() {
        let file_content = r#"
            export { foo } from '@src/foo';

            export {
                test,
            }

             export {
                value,
            };

            export {
                test2,
                test3, test4,
            } from '@src/foo';
        "#;
        let imports = extract_imports_from_file_content(file_content).unwrap();

        assert_eq!(
            imports,
            vec![
                Import {
                    import_path: PathBuf::from("@src/foo"),
                    line: 2,
                    values: ImportType::Named(vec!["foo".to_string()]),
                },
                Import {
                    import_path: PathBuf::from("@src/foo"),
                    line: 12,
                    values: ImportType::Named(vec![
                        "test2".to_string(),
                        "test3".to_string(),
                        "test4".to_string(),
                    ]),
                }
            ],
        );
    }
}
