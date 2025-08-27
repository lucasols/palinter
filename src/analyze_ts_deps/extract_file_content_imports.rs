use lazy_static::lazy_static;
use regex::Regex;
use std::path::PathBuf;

use crate::utils::{get_code_from_line, remove_comments_from_code};

#[derive(Debug, PartialEq, Clone)]
pub enum ImportType {
    Named(Vec<String>),
    All,
    Dynamic,
    Type(Vec<String>),
    SideEffect,
    Glob,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Import {
    pub import_path: PathBuf,
    pub line: usize,
    pub values: ImportType,
}

const DEFAULT: &str = "default";

pub fn extract_imports_from_file_content(
    file_content: &str,
) -> Result<Vec<Import>, String> {
    let file_content = remove_comments_from_code(file_content);

    let mut imports = Vec::new();

    let mut current_line = 0;
    let lines_iter = file_content.lines();
    let lines = lines_iter.clone().collect::<Vec<&str>>();

    while current_line < lines.len() {
        current_line += 1;

        let line = lines[current_line - 1].trim();

        lazy_static! {
            static ref IMPORTS_RE: Regex = Regex::new(
                r#"(?x)
                    ^(?:import|export)\s+
                    (?:
                        (?P<all>
                            \*.+|
                            [$\w]+\s*,\s+\*.+
                        )
                        |
                        \{(?P<named>[$\w\s,]+?)\}\s+
                        |
                        (?P<default>[$\w]+\s+)
                        |
                        [$\w]+\s*,\s+\{(?P<named_with_default>[$\w\s,]+?)\}\s+
                        |
                        type\s+\{(?P<types>[$\w\s,]+?)\}\s+
                    )
                    from\s+["'](?P<import_path>.+)["']
                "#
            )
            .unwrap();
            static ref SIDE_EFFECT_IMPORT_RE: Regex = Regex::new(
                r#"(?x)
                    ^import\s+
                    ["'](?P<import_path>.+)["']
                "#
            )
            .unwrap();
        }

        if line.starts_with("import") || line.starts_with("export") {
            if let Some(captures) = SIDE_EFFECT_IMPORT_RE.captures(line) {
                let import_path =
                    captures.name("import_path").unwrap().as_str().to_string();

                imports.push(Import {
                    import_path: PathBuf::from(import_path),
                    line: current_line,
                    values: ImportType::SideEffect,
                });
                continue;
            }

            let content_to_check = if line.contains("from") {
                line.to_string()
            } else {
                get_code_from_line(&lines_iter, current_line)
            };

            if let Some(captures) = IMPORTS_RE.captures(&content_to_check) {
                let full_match_lines =
                    captures.get(0).unwrap().as_str().lines().count();

                let import_path =
                    captures.name("import_path").unwrap().as_str().to_string();

                if captures.name("all").is_some() {
                    imports.push(Import {
                        import_path: PathBuf::from(import_path),
                        line: current_line,
                        values: ImportType::All,
                    });
                } else if let Some(values_string) = captures.name("named") {
                    let values = values_string
                        .as_str()
                        .split(',')
                        .filter_map(filter_map_named_import_value)
                        .collect::<Vec<String>>();

                    imports.push(Import {
                        import_path: PathBuf::from(import_path),
                        line: current_line,
                        values: ImportType::Named(values),
                    });
                } else if captures.name("default").is_some() {
                    imports.push(Import {
                        import_path: PathBuf::from(import_path),
                        line: current_line,
                        values: ImportType::Named(vec![DEFAULT.to_string()]),
                    });
                } else if let Some(values_string) =
                    captures.name("named_with_default")
                {
                    let mut values = values_string
                        .as_str()
                        .split(',')
                        .filter_map(filter_map_named_import_value)
                        .collect::<Vec<String>>();

                    values.push(DEFAULT.to_string());

                    imports.push(Import {
                        import_path: PathBuf::from(import_path),
                        line: current_line,
                        values: ImportType::Named(values),
                    });
                } else if let Some(type_imports) = captures.name("types") {
                    let values = type_imports
                        .as_str()
                        .split(',')
                        .filter_map(filter_map_named_import_value)
                        .collect::<Vec<String>>();

                    imports.push(Import {
                        import_path: PathBuf::from(import_path),
                        line: current_line,
                        values: ImportType::Type(values),
                    });
                }

                current_line += full_match_lines - 1;
            }
        } else {
            lazy_static! {
                static ref DYNAMIC_IMPORT: Regex =
                    Regex::new(r#"import\(\s*['"](.+?)['"]\s*\)"#).unwrap();
                static ref IS_MULTILINE_DYNAMIC_IMPORT: Regex =
                    Regex::new(r#"(import|glob)\(\s*$"#).unwrap();
                static ref GLOB_IMPORT: Regex =
                    Regex::new(r#"import\.meta\.glob\(\s*['"](.+?)['"]"#).unwrap();
            }

            if line.starts_with(r"\\") {
                continue;
            }

            let content_to_check = if IS_MULTILINE_DYNAMIC_IMPORT.is_match(line) {
                get_code_from_line(&lines_iter, current_line)
            } else {
                line.to_string()
            };

            if let Some(captures) = DYNAMIC_IMPORT.captures(&content_to_check) {
                let full_match_lines =
                    captures.get(0).unwrap().as_str().lines().count();

                let import_path = captures.get(1).unwrap().as_str().to_string();

                imports.push(Import {
                    import_path: PathBuf::from(import_path),
                    line: current_line,
                    values: ImportType::Dynamic,
                });

                current_line += full_match_lines - 1;
            }

            if let Some(captures) = GLOB_IMPORT.captures(&content_to_check) {
                let full_match_lines =
                    captures.get(0).unwrap().as_str().lines().count();

                let import_path = captures.get(1).unwrap().as_str().to_string();

                imports.push(Import {
                    import_path: PathBuf::from(&import_path),
                    line: current_line,
                    values: ImportType::Glob,
                });

                current_line += full_match_lines - 1;
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

    let non_type_value = value.replace("type ", "");

    if non_type_value.is_empty() {
        None
    } else {
        Some(non_type_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;
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
        // comment
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
                    line: 3,
                    values: ImportType::Named(vec![
                        "Foo".to_string(),
                        "Bar".to_string()
                    ])
                },
                Import {
                    import_path: PathBuf::from("@src/baz"),
                    line: 4,
                    values: ImportType::Named(vec!["Baz".to_string()]),
                },
                Import {
                    import_path: PathBuf::from("@src/qux"),
                    line: 5,
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

            // const test2 = import('@src/foo');
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

    #[test]
    fn side_effect_import() {
        let file_content = r#"
            import '@src/foo';
        "#;
        let imports = extract_imports_from_file_content(file_content).unwrap();

        assert_eq!(
            imports,
            vec![Import {
                import_path: PathBuf::from("@src/foo"),
                line: 2,
                values: ImportType::SideEffect,
            }],
        );
    }

    #[test]
    fn bug_test() {
        let file_content = r#"
import { apiMutation } from '@src/api/apiCall';
import { validateApiResponse } from '@src/api/apiStores/apiStores.utils';
import { ChatType } from '@src/api/schemas/resources/conversationResource';
import { getCurrentUserId } from '@src/state/userStore';
import { chatMessagesList } from '@src/stores/chat/chatMessagesList';
import {
ChatConversation,
chatsList,
conversationListMetaId,
conversationSchema,
convertFromApiConversation,
} from '@src/stores/chat/chatsList';
import { isBetaFeature } from '@src/utils/betaFeatures';
import { navigate } from '@src/utils/history';
import { strictAssertIsNotNullish } from '@utils/typeAssertions';
"#;

        let imports = extract_imports_from_file_content(file_content).unwrap();

        assert_debug_snapshot!(
            imports,
            @r###"
        [
            Import {
                import_path: "@src/api/apiCall",
                line: 2,
                values: Named(
                    [
                        "apiMutation",
                    ],
                ),
            },
            Import {
                import_path: "@src/api/apiStores/apiStores.utils",
                line: 3,
                values: Named(
                    [
                        "validateApiResponse",
                    ],
                ),
            },
            Import {
                import_path: "@src/api/schemas/resources/conversationResource",
                line: 4,
                values: Named(
                    [
                        "ChatType",
                    ],
                ),
            },
            Import {
                import_path: "@src/state/userStore",
                line: 5,
                values: Named(
                    [
                        "getCurrentUserId",
                    ],
                ),
            },
            Import {
                import_path: "@src/stores/chat/chatMessagesList",
                line: 6,
                values: Named(
                    [
                        "chatMessagesList",
                    ],
                ),
            },
            Import {
                import_path: "@src/stores/chat/chatsList",
                line: 7,
                values: Named(
                    [
                        "ChatConversation",
                        "chatsList",
                        "conversationListMetaId",
                        "conversationSchema",
                        "convertFromApiConversation",
                    ],
                ),
            },
            Import {
                import_path: "@src/utils/betaFeatures",
                line: 14,
                values: Named(
                    [
                        "isBetaFeature",
                    ],
                ),
            },
            Import {
                import_path: "@src/utils/history",
                line: 15,
                values: Named(
                    [
                        "navigate",
                    ],
                ),
            },
            Import {
                import_path: "@utils/typeAssertions",
                line: 16,
                values: Named(
                    [
                        "strictAssertIsNotNullish",
                    ],
                ),
            },
        ]
        "###
        );
    }

    #[test]
    fn bug_test_2() {
        let file_content = r#"
import {
  ShellMenuItemTabIcon,
  ShellMenuItem,
} from '@src/components/shellMenu/ShellMenuItem';
import { ShellMenuItemsContainer } from '@src/components/shellMenu/ShellMenuItemsContainer';
import { ShellMenuItemsGroup } from '@src/components/shellMenu/ShellMenuItemsGroup';
import { ShellMenuLoadingState } from '@src/components/shellMenu/ShellMenuLoadingState';
import { UserProfilePicWithStatus } from '@src/components/UserProfilePicWithStatus';
import { getCurrentUserId } from '@src/state/userStore';
import {
  ChatConversation,
  chatsList,
  UseListQueryConversations,
} from '@src/stores/chat/chatsList';
import { createDm } from '@src/stores/chat/chatsList.actions';
import { NormalizedTab } from '@src/stores/tabsCollection';
import { useUsers } from '@src/stores/usersList';
import { usersStatusStore } from '@src/stores/usersStatusStore';
import { getChatPath, parseRecordChatSlug } from '@src/utils/chat.utils';
import { navigate } from '@src/utils/history';
import { useMutationWithValueState } from '@src/utils/hooks/useSavingState';
import { localSync, useLocalSync } from '@src/utils/localSync';
import type { useArrayToMap } from '@utils/hooks/useArrayToMap';
import { __ } from '@utils/i18n/i18n';
import type { isNotNullish, isNullish } from '@utils/isNullish';
import { sortBy } from '@utils/sortBy';
import type { useMemo } from 'react';
import { useStoreSnapshot } from 't-state';
"#;

        let imports = extract_imports_from_file_content(file_content).unwrap();

        assert_debug_snapshot!(
            imports,
            @r###"
        [
            Import {
                import_path: "@src/components/shellMenu/ShellMenuItem",
                line: 2,
                values: Named(
                    [
                        "ShellMenuItemTabIcon",
                        "ShellMenuItem",
                    ],
                ),
            },
            Import {
                import_path: "@src/components/shellMenu/ShellMenuItemsContainer",
                line: 6,
                values: Named(
                    [
                        "ShellMenuItemsContainer",
                    ],
                ),
            },
            Import {
                import_path: "@src/components/shellMenu/ShellMenuItemsGroup",
                line: 7,
                values: Named(
                    [
                        "ShellMenuItemsGroup",
                    ],
                ),
            },
            Import {
                import_path: "@src/components/shellMenu/ShellMenuLoadingState",
                line: 8,
                values: Named(
                    [
                        "ShellMenuLoadingState",
                    ],
                ),
            },
            Import {
                import_path: "@src/components/UserProfilePicWithStatus",
                line: 9,
                values: Named(
                    [
                        "UserProfilePicWithStatus",
                    ],
                ),
            },
            Import {
                import_path: "@src/state/userStore",
                line: 10,
                values: Named(
                    [
                        "getCurrentUserId",
                    ],
                ),
            },
            Import {
                import_path: "@src/stores/chat/chatsList",
                line: 11,
                values: Named(
                    [
                        "ChatConversation",
                        "chatsList",
                        "UseListQueryConversations",
                    ],
                ),
            },
            Import {
                import_path: "@src/stores/chat/chatsList.actions",
                line: 16,
                values: Named(
                    [
                        "createDm",
                    ],
                ),
            },
            Import {
                import_path: "@src/stores/tabsCollection",
                line: 17,
                values: Named(
                    [
                        "NormalizedTab",
                    ],
                ),
            },
            Import {
                import_path: "@src/stores/usersList",
                line: 18,
                values: Named(
                    [
                        "useUsers",
                    ],
                ),
            },
            Import {
                import_path: "@src/stores/usersStatusStore",
                line: 19,
                values: Named(
                    [
                        "usersStatusStore",
                    ],
                ),
            },
            Import {
                import_path: "@src/utils/chat.utils",
                line: 20,
                values: Named(
                    [
                        "getChatPath",
                        "parseRecordChatSlug",
                    ],
                ),
            },
            Import {
                import_path: "@src/utils/history",
                line: 21,
                values: Named(
                    [
                        "navigate",
                    ],
                ),
            },
            Import {
                import_path: "@src/utils/hooks/useSavingState",
                line: 22,
                values: Named(
                    [
                        "useMutationWithValueState",
                    ],
                ),
            },
            Import {
                import_path: "@src/utils/localSync",
                line: 23,
                values: Named(
                    [
                        "localSync",
                        "useLocalSync",
                    ],
                ),
            },
            Import {
                import_path: "@utils/hooks/useArrayToMap",
                line: 24,
                values: Type(
                    [
                        "useArrayToMap",
                    ],
                ),
            },
            Import {
                import_path: "@utils/i18n/i18n",
                line: 25,
                values: Named(
                    [
                        "__",
                    ],
                ),
            },
            Import {
                import_path: "@utils/isNullish",
                line: 26,
                values: Type(
                    [
                        "isNotNullish",
                        "isNullish",
                    ],
                ),
            },
            Import {
                import_path: "@utils/sortBy",
                line: 27,
                values: Named(
                    [
                        "sortBy",
                    ],
                ),
            },
            Import {
                import_path: "react",
                line: 28,
                values: Type(
                    [
                        "useMemo",
                    ],
                ),
            },
            Import {
                import_path: "t-state",
                line: 29,
                values: Named(
                    [
                        "useStoreSnapshot",
                    ],
                ),
            },
        ]
        "###
        );
    }

    #[test]
    fn glob_import() {
        let file_content = r#"
            const test = import.meta.glob('./test/*.ts');

            const globEager = import.meta.glob('./test_2/*.ts', { eager: true });
        "#;
        let imports = extract_imports_from_file_content(file_content).unwrap();

        assert_debug_snapshot!(
            imports,
            @r###"
        [
            Import {
                import_path: "./test/*.ts",
                line: 2,
                values: Glob,
            },
            Import {
                import_path: "./test_2/*.ts",
                line: 4,
                values: Glob,
            },
        ]
        "###
        );
    }

    #[test]
    fn glob_import_2() {
        let file_content = r#"

const compactFieldComponents = import.meta.glob(
    '/src/tableFields/fields/*/Compact*Field.tsx',
    { eager: true },
);

const tableBodyCellViewerComponents = import.meta.glob(
    '/src/tableFields/fields/*/Table*CellViewer.tsx',
    { eager: true },
);
"#;

        let imports = extract_imports_from_file_content(file_content).unwrap();

        assert_debug_snapshot!(
            imports,
            @r###"
        [
            Import {
                import_path: "/src/tableFields/fields/*/Compact*Field.tsx",
                line: 3,
                values: Glob,
            },
            Import {
                import_path: "/src/tableFields/fields/*/Table*CellViewer.tsx",
                line: 8,
                values: Glob,
            },
        ]
        "###
        );
    }

    #[test]
    fn named_type_import() {
        let file_content = r#"
          import { a, type Test, type Test2 as Test3 } from '@src/foo';
        "#;
        let imports = extract_imports_from_file_content(file_content).unwrap();

        assert_eq!(
            imports,
            vec![Import {
                import_path: PathBuf::from("@src/foo"),
                line: 2,
                values: ImportType::Named(vec![
                    "a".to_string(),
                    "Test".to_string(),
                    "Test2".to_string()
                ]),
            }],
        );
    }

    #[test]
    fn import_function_with_dollar_sign() {
        let file_content = r#"
            import { $imageNode, normalFunction } from '@src/lexical';
        "#;
        let imports = extract_imports_from_file_content(file_content).unwrap();

        assert_eq!(
            imports,
            vec![Import {
                import_path: PathBuf::from("@src/lexical"),
                line: 2,
                values: ImportType::Named(vec![
                    "$imageNode".to_string(),
                    "normalFunction".to_string(),
                ]),
            }],
        );
    }
}
