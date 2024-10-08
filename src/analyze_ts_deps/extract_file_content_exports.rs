use lazy_static::lazy_static;
use regex::Regex;

use crate::utils::{get_code_from_line, remove_comments_from_code, split_string_by};

#[derive(Debug, PartialEq, Clone)]
pub struct Export {
    pub line: usize,
    pub name: String,
    pub ignored: bool,
}

const DEFAULT: &str = "default";

pub fn extract_file_content_exports(
    file_content: &str,
) -> Result<Vec<Export>, String> {
    let file_content = remove_comments_from_code(file_content);

    let mut exports = Vec::new();

    let mut current_line = 0;
    let lines_iter = file_content.lines();
    let lines = lines_iter.clone().collect::<Vec<&str>>();

    let mut ignore_exports_in_line = 0;

    while current_line < lines.len() {
        current_line += 1;

        let line = lines[current_line - 1].trim();

        if line.starts_with("// palinter-ignore-unused-next-line") {
            ignore_exports_in_line = current_line + 1;
            continue;
        }

        if line.starts_with("export default") {
            exports.push(Export {
                line: current_line,
                name: DEFAULT.to_string(),
                ignored: ignore_exports_in_line == current_line,
            });
        } else {
            lazy_static! {
                static ref SIMPLE_EXPORT: Regex = Regex::new(
                    r#"^export\s+(let|const|class|function\*?|async\s+?function\*?|type|interface)\s+(\w+)"#
                )
                .unwrap();

                static ref CAN_BE_VALUE_MULTILINE_EXPORT: Regex = Regex::new(
                    r#"^export\s+(let|const)"#
                ).unwrap();

                static ref CAN_BE_MULTILINE_EXPORT: Regex = Regex::new(
                    r#"^export\s+\{"#
                ).unwrap();

                static ref DESTRUCTURED_VALUE_EXPORT: Regex = Regex::new(
                   r#"^export\s+(let|const)\s+[\{\[]([\S\s]+?)[\}\]]"#
                ).unwrap();

                static ref DESTRUCTURED_EXPORT: Regex = Regex::new(
                    r#"^export\s+\{([\S\s]+?)\}"#
                ).unwrap();
            }

            if let Some(captures) = SIMPLE_EXPORT.captures(line) {
                exports.push(Export {
                    line: current_line,
                    name: captures.get(2).unwrap().as_str().to_string(),
                    ignored: ignore_exports_in_line == current_line,
                });
                continue;
            }

            if CAN_BE_VALUE_MULTILINE_EXPORT.is_match(line) {
                let remaining_file_content =
                    get_code_from_line(&lines_iter, current_line);

                if let Some(captures) =
                    DESTRUCTURED_VALUE_EXPORT.captures(&remaining_file_content)
                {
                    let destructured_values =
                        captures.get(2).unwrap().as_str().to_string();

                    let full_match_lines =
                        captures.get(0).unwrap().as_str().lines().count();

                    for destructured_value in destructured_values.split(',') {
                        let destructured_value = destructured_value.trim();

                        let name_part =
                            match split_string_by(destructured_value, "=") {
                                Some((name_part, _)) => name_part.trim().to_string(),
                                None => destructured_value.to_string(),
                            };

                        let name = match split_string_by(&name_part, ":") {
                            Some((_, name)) => name.trim().to_string(),
                            None => name_part,
                        };

                        if name.is_empty() {
                            continue;
                        }

                        exports.push(Export {
                            line: current_line,
                            name,
                            ignored: ignore_exports_in_line == current_line,
                        });
                    }

                    current_line += full_match_lines - 1;
                    continue;
                }
            } else if CAN_BE_MULTILINE_EXPORT.is_match(line) {
                let remaining_file_content =
                    get_code_from_line(&lines_iter, current_line);

                if let Some(captures) =
                    DESTRUCTURED_EXPORT.captures(&remaining_file_content)
                {
                    let destructured_values =
                        captures.get(1).unwrap().as_str().to_string();

                    let full_match_lines =
                        captures.get(0).unwrap().as_str().lines().count();

                    for destructured_value in destructured_values.split(',') {
                        let destructured_value = destructured_value.trim();

                        let name = match split_string_by(destructured_value, "as") {
                            Some((_, name)) => name.trim().to_string(),
                            None => destructured_value.to_string(),
                        };

                        exports.push(Export {
                            line: current_line,
                            name,
                            ignored: ignore_exports_in_line == current_line,
                        });
                    }

                    current_line += full_match_lines - 1;
                    continue;
                }
            }
        }
    }

    Ok(exports)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(test)]
    use pretty_assertions::assert_eq;

    #[test]
    fn default_export() {
        let file_content = r#"
          export default function foo() {}
        "#;
        let exports = extract_file_content_exports(file_content).unwrap();
        assert_eq!(
            exports,
            vec![Export {
                line: 2,
                name: DEFAULT.to_string(),
                ignored: false,
            }],
        );
    }

    #[test]
    fn const_exports() {
        let file_content = r#"
          export const foo = "bar";
          export const bar = "baz";
          export const baz = "foo";
        "#;
        let exports = extract_file_content_exports(file_content).unwrap();
        assert_eq!(
            exports,
            vec![
                Export {
                    line: 2,
                    name: "foo".to_string(),
                    ignored: false,
                },
                Export {
                    line: 3,
                    name: "bar".to_string(),
                    ignored: false,
                },
                Export {
                    line: 4,
                    name: "baz".to_string(),
                    ignored: false,
                },
            ]
        );
    }

    #[test]
    fn exporting_declarations() {
        let file_content = r#"
          export const foo = "bar";
          export function bar() {}
          export function* bar2() {}
          export class baz {}
          export type Test = ''
          export let test3 = ''
          export interface Test2 {}
          export async function test4() {}
          export async function* test5() {}
        "#;
        let exports = extract_file_content_exports(file_content).unwrap();
        assert_eq!(
            exports,
            vec![
                Export {
                    line: 2,
                    name: "foo".to_string(),
                    ignored: false,
                },
                Export {
                    line: 3,
                    name: "bar".to_string(),
                    ignored: false,
                },
                Export {
                    line: 4,
                    name: "bar2".to_string(),
                    ignored: false,
                },
                Export {
                    line: 5,
                    name: "baz".to_string(),
                    ignored: false,
                },
                Export {
                    line: 6,
                    name: "Test".to_string(),
                    ignored: false,
                },
                Export {
                    line: 7,
                    name: "test3".to_string(),
                    ignored: false,
                },
                Export {
                    line: 8,
                    name: "Test2".to_string(),
                    ignored: false,
                },
                Export {
                    line: 9,
                    name: "test4".to_string(),
                    ignored: false,
                },
                Export {
                    line: 10,
                    name: "test5".to_string(),
                    ignored: false,
                },
            ]
        );
    }

    #[test]
    fn exporting_declarations_2() {
        let file_content = r#"
          export const { name1, name2: bar } = o;
          export const [ name2, name3 ] = array;
          export let [test ] = array;
          export let [test2 = 1 ] = array;
          export let {test3 = 1, test: test4 = 2 } = obj;
        "#;
        let exports = extract_file_content_exports(file_content).unwrap();

        assert_eq!(
            exports,
            vec![
                Export {
                    line: 2,
                    name: "name1".to_string(),
                    ignored: false,
                },
                Export {
                    line: 2,
                    name: "bar".to_string(),
                    ignored: false,
                },
                Export {
                    line: 3,
                    name: "name2".to_string(),
                    ignored: false,
                },
                Export {
                    line: 3,
                    name: "name3".to_string(),
                    ignored: false,
                },
                Export {
                    line: 4,
                    name: "test".to_string(),
                    ignored: false,
                },
                Export {
                    line: 5,
                    name: "test2".to_string(),
                    ignored: false,
                },
                Export {
                    line: 6,
                    name: "test3".to_string(),
                    ignored: false,
                },
                Export {
                    line: 6,
                    name: "test4".to_string(),
                    ignored: false,
                },
            ]
        );
    }

    #[test]
    fn exporting_declarations_3() {
        let file_content = r#"
           export const { name1,
            name2: bar
             } = o;
          export const [
            name2,
            name3 ] = array;
        "#;
        let exports = extract_file_content_exports(file_content).unwrap();
        assert_eq!(
            exports,
            vec![
                Export {
                    line: 2,
                    name: "name1".to_string(),
                    ignored: false,
                },
                Export {
                    line: 2,
                    name: "bar".to_string(),
                    ignored: false,
                },
                Export {
                    line: 5,
                    name: "name2".to_string(),
                    ignored: false,
                },
                Export {
                    line: 5,
                    name: "name3".to_string(),
                    ignored: false,
                },
            ]
        );
    }

    #[test]
    fn export_list() {
        let file_content = r#"
          export { foo };
        "#;

        let exports = extract_file_content_exports(file_content).unwrap();

        assert_eq!(
            exports,
            vec![Export {
                line: 2,
                name: "foo".to_string(),
                ignored: false,
            }],
        );
    }

    #[test]
    fn export_list_multiline() {
        let file_content = r#"
          export { foo,
            bar,
            baz as test
           };
        "#;

        let exports = extract_file_content_exports(file_content).unwrap();

        assert_eq!(
            exports,
            vec![
                Export {
                    line: 2,
                    name: "foo".to_string(),
                    ignored: false,
                },
                Export {
                    line: 2,
                    name: "bar".to_string(),
                    ignored: false,
                },
                Export {
                    line: 2,
                    name: "test".to_string(),
                    ignored: false,
                }
            ],
        );
    }

    #[test]
    fn reexport() {
        let file_content = r#"
          export { foo } from './foo';
          export { foo as test } from './foo2';
          export { default } from './foo2';
        "#;
        let exports = extract_file_content_exports(file_content).unwrap();

        assert_eq!(
            exports,
            vec![
                Export {
                    line: 2,
                    name: "foo".to_string(),
                    ignored: false,
                },
                Export {
                    line: 3,
                    name: "test".to_string(),
                    ignored: false,
                },
                Export {
                    line: 4,
                    name: DEFAULT.to_string(),
                    ignored: false,
                }
            ],
        );
    }

    #[test]
    fn ignore_unused_next_line() {
        let file_content = r#"
          export const foo = "bar";
          // palinter-ignore-unused-next-line
          export const bar = "baz";
        "#;
        let exports = extract_file_content_exports(file_content).unwrap();
        assert_eq!(
            exports,
            vec![
                Export {
                    line: 2,
                    name: "foo".to_string(),
                    ignored: false,
                },
                Export {
                    line: 4,
                    name: "bar".to_string(),
                    ignored: true,
                }
            ],
        );
    }

    #[test]
    fn extract_export_bug() {
        let file_content = r#"
            export const {
                getCurrentStackedModals,
                useStackedModals,
                preventClosingOfStackedModals,
                stackedModalEvents,
                closeModal: closeStackedModal,
            } = stackedModalsMethods;
        "#;
        let exports = extract_file_content_exports(file_content).unwrap();

        assert_eq!(
            exports,
            vec![
                Export {
                    line: 2,
                    name: "getCurrentStackedModals".to_string(),
                    ignored: false,
                },
                Export {
                    line: 2,
                    name: "useStackedModals".to_string(),
                    ignored: false,
                },
                Export {
                    line: 2,
                    name: "preventClosingOfStackedModals".to_string(),
                    ignored: false,
                },
                Export {
                    line: 2,
                    name: "stackedModalEvents".to_string(),
                    ignored: false,
                },
                Export {
                    line: 2,
                    name: "closeStackedModal".to_string(),
                    ignored: false,
                }
            ],
        );
    }
}
