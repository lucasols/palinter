use std::collections::BTreeMap;

use crate::test_utils::TEST_MUTEX;

use super::*;
use insta::assert_debug_snapshot;

struct SimplifiedFile {
    path: PathBuf,
    content: String,
}

fn get_results(
    files: Vec<SimplifiedFile>,
    entry_point: &str,
) -> (BTreeMap<String, DepsResult>, BTreeMap<String, FileDepsInfo>) {
    let mut results = BTreeMap::new();

    let flatten_root_structure = create_flatten_root_structure(files);

    get_used_project_files_deps_info(
        [PathBuf::from(entry_point)].to_vec(),
        flatten_root_structure.clone(),
        HashMap::from_iter(vec![(String::from("@src"), String::from("./src"))]),
    )
    .unwrap();

    for (file_path, _) in BTreeMap::from_iter(flatten_root_structure) {
        let deps_info =
            get_file_deps_result(&PathBuf::from(file_path.clone())).unwrap();

        let mut sorted_deps: Vec<String> = deps_info.deps.iter().cloned().collect();

        sorted_deps.sort();

        let deps_info_with_sorted_deps = DepsResult {
            deps: sorted_deps.into_iter().collect(),
            circular_deps: deps_info.circular_deps.clone(),
        };

        results.insert(file_path, deps_info_with_sorted_deps);
    }

    (
        results,
        BTreeMap::from_iter(USED_FILES.lock().unwrap().clone()),
    )
}

fn create_flatten_root_structure(
    files: Vec<SimplifiedFile>,
) -> HashMap<String, File> {
    let mut flatten_root_structure = HashMap::new();

    for file in files {
        let file_to_add = File {
            basename: file.path.file_stem().unwrap().to_str().unwrap().to_string(),
            name_with_ext: file
                .path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            content: Some(file.content),
            extension: None,
            relative_path: file.path.to_str().unwrap().to_string(),
        };

        flatten_root_structure
            .insert(file.path.to_str().unwrap().to_string(), file_to_add);
    }

    flatten_root_structure
}

#[test]
fn get_project_files_deps_info_test() {
    let _guard = TEST_MUTEX.lock().unwrap();

    _setup_test();

    let results = get_results(
        vec![
            SimplifiedFile {
                path: PathBuf::from("./src/index.ts"),
                content: String::from("import '@src/fileB';"),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileB.ts"),
                content: String::from(
                    r#"
                    import { a } from '@src/fileA';
                    import { b } from '@src/fileA';
                    import { test } from 'testLib';
                    import '@src/img.svg';
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileA.ts"),
                content: String::from(
                    r#"
                    export const a = 1;
                    export const b = 2;
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/img.svg"),
                content: "".to_string(),
            },
        ],
        "@src/index.ts",
    );

    assert_debug_snapshot!(results);
}

#[test]
fn get_project_files_deps_info_test_2() {
    let _guard = TEST_MUTEX.lock().unwrap();

    _setup_test();

    let (results, _) = get_results(
        vec![
            SimplifiedFile {
                path: PathBuf::from("./src/index.ts"),
                content: String::from(
                    r#"
                    import { a } from '@src/fileA';
                    import { b } from '@src/fileB';
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileB.ts"),
                content: String::from(
                    r#"
                    import { c } from '@src/fileC';
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileA.ts"),
                content: String::from(
                    r#"
                    import { c } from '@src/fileC';
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileC.ts"),
                content: String::from(
                    r#"
                    export const c = 1;
                    "#,
                ),
            },
        ],
        "@src/index.ts",
    );

    assert_debug_snapshot!(results);
}

#[test]
fn project_with_circular_deps() {
    let _guard = TEST_MUTEX.lock().unwrap();

    _setup_test();

    let (results, _) = get_results(
        vec![
            SimplifiedFile {
                path: PathBuf::from("./src/index.ts"),
                content: String::from("import '@src/fileB';"),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileB.ts"),
                content: String::from(
                    r#"
                    import { a } from '@src/fileA';
                    import { b } from '@src/fileA';
                    import { test } from 'testLib';
                    export const c = 2;
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileA.ts"),
                content: String::from(
                    r#"
                    import { c } from '@src/fileB';
                    export const a = 1;
                    export const b = 2;
                    "#,
                ),
            },
        ],
        "@src/index.ts",
    );

    assert_debug_snapshot!(results);
}

fn get_file(name: &str, files: &[&str]) -> SimplifiedFile {
    let content = files
        .iter()
        .map(|file| format!("import '@src/{}.ts';", file))
        .collect::<Vec<String>>()
        .join("\n");

    SimplifiedFile {
        path: PathBuf::from(format!("./src/{}.ts", name)),
        content,
    }
}

#[test]
fn project_with_circular_deps_2() {
    let _guard = TEST_MUTEX.lock().unwrap();

    _setup_test();

    let (results, _) = get_results(
        vec![
            get_file("index", &["b", "c"]),
            get_file("c", &["d"]),
            get_file("b", &["d", "e"]),
            get_file("d", &["1"]),
            get_file("e", &["f"]),
            get_file("f", &["b"]),
            get_file("1", &["2"]),
            get_file("2", &["3"]),
            get_file("3", &["1"]),
        ],
        "@src/index.ts",
    );

    assert_debug_snapshot!(results);
}

#[test]
fn project_with_circular_deps_3() {
    let _guard = TEST_MUTEX.lock().unwrap();

    _setup_test();

    let (results, _) = get_results(
        vec![
            get_file("a", &["b", "c", "p"]),
            get_file("b", &["c", "d"]),
            get_file("c", &["b", "d", "e"]),
            get_file("d", &["b", "c", "e", "f"]),
            get_file("e", &["c", "d", "f", "g"]),
            get_file("f", &["d", "e", "g", "h"]),
            get_file("g", &["e", "f", "h", "i"]),
            get_file("h", &["f", "g", "i", "j"]),
            get_file("i", &["g", "h", "j", "k"]),
            get_file("j", &["h", "i", "k", "l"]),
            get_file("k", &["i", "j", "l", "m"]),
            get_file("l", &["j", "k", "m", "n"]),
            get_file("m", &["k", "l", "n", "o"]),
            get_file("n", &["l", "m", "o", "p"]),
            get_file("o", &["m", "n", "p", "b"]),
            get_file("p", &["n", "o", "c"]),
        ],
        "@src/a.ts",
    );

    assert_debug_snapshot!(results);
}

#[test]
fn project_with_circular_deps_4() {
    let _guard = TEST_MUTEX.lock().unwrap();

    _setup_test();

    let (results, _) = get_results(
        vec![
            get_file("a", &["b", "c"]),
            get_file("b", &["c", "d"]),
            get_file("c", &["b", "d"]),
            get_file("d", &[]),
        ],
        "@src/a.ts",
    );

    assert_debug_snapshot!(results);
}

#[test]
fn project_with_glob_imports() {
    let _guard = TEST_MUTEX.lock().unwrap();

    _setup_test();

    let (results, _) = get_results(
        vec![
            SimplifiedFile {
                path: PathBuf::from("./src/index.ts"),
                content: String::from(
                    r#"
                    import { a } from '@src/fileA';
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileA.ts"),
                content: String::from(
                    r#"
                    const test = import.meta.glob('/src/file_*.ts');
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/file_B.ts"),
                content: String::from(
                    r#"
                    export const b = 2;
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/file_C.ts"),
                content: String::from(
                    r#"
                    export const c = 2;
                    "#,
                ),
            },
        ],
        "@src/index.ts",
    );

    assert_debug_snapshot!(results, @r###"
    {
        "./src/fileA.ts": DepsResult {
            deps: {
                "./src/file_B.ts",
                "./src/file_C.ts",
            },
            circular_deps: None,
        },
        "./src/file_B.ts": DepsResult {
            deps: {},
            circular_deps: None,
        },
        "./src/file_C.ts": DepsResult {
            deps: {},
            circular_deps: None,
        },
        "./src/index.ts": DepsResult {
            deps: {
                "./src/fileA.ts",
                "./src/file_B.ts",
                "./src/file_C.ts",
            },
            circular_deps: None,
        },
    }
    "###);
}

#[test]
fn project_with_glob_imports_and_circular_deps() {
    let _guard = TEST_MUTEX.lock().unwrap();

    _setup_test();

    let (results, _) = get_results(
        vec![
            SimplifiedFile {
                path: PathBuf::from("./src/index.ts"),
                content: String::from(
                    r#"
                    import { a } from '@src/fileA';
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileA.ts"),
                content: String::from(
                    r#"
                    const test = import.meta.glob('/src/file_*.ts');
                    export const a = 1;
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/file_B.ts"),
                content: String::from(
                    r#"
                    console.log('b');
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/file_C.ts"),
                content: String::from(
                    r#"
                    import { b } from '@src/fileA';
                    "#,
                ),
            },
        ],
        "@src/index.ts",
    );

    assert_debug_snapshot!(results, @r###"
    {
        "./src/fileA.ts": DepsResult {
            deps: {
                "./src/fileA.ts",
                "./src/file_B.ts",
                "./src/file_C.ts",
            },
            circular_deps: Some(
                [
                    "./src/fileA.ts",
                ],
            ),
        },
        "./src/file_B.ts": DepsResult {
            deps: {},
            circular_deps: None,
        },
        "./src/file_C.ts": DepsResult {
            deps: {
                "./src/fileA.ts",
                "./src/file_B.ts",
                "./src/file_C.ts",
            },
            circular_deps: Some(
                [
                    "./src/fileA.ts",
                ],
            ),
        },
        "./src/index.ts": DepsResult {
            deps: {
                "./src/fileA.ts",
                "./src/file_B.ts",
                "./src/file_C.ts",
            },
            circular_deps: Some(
                [
                    "./src/fileA.ts",
                ],
            ),
        },
    }
    "###);
}

#[test]
fn normalize_imports_preserves_type_and_named_separately() {
    let _guard = TEST_MUTEX.lock().unwrap();

    _setup_test();

    let (_, used_files) = get_results(
        vec![
            SimplifiedFile {
                path: PathBuf::from("./src/index.ts"),
                content: String::from(
                    r#"
                    import { foo } from '@src/fileA';
                    import type { Bar } from '@src/fileA';
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileA.ts"),
                content: String::from(
                    r#"
                    export const foo = 1;
                    export type Bar = string;
                    "#,
                ),
            },
        ],
        "@src/index.ts",
    );

    let index_info = used_files.get("./src/index.ts").unwrap();
    let imports_to_file_a = index_info.imports.get("./src/fileA.ts").unwrap();

    let has_named = imports_to_file_a.iter().any(|i| {
        matches!(&i.values, ImportType::Named(v) if v.contains(&"foo".to_string()))
    });
    let has_type = imports_to_file_a.iter().any(|i| {
        matches!(&i.values, ImportType::Type(v) if v.contains(&"Bar".to_string()))
    });

    assert!(
        has_named,
        "Should have a Named import with 'foo', got: {:?}",
        imports_to_file_a
    );
    assert!(
        has_type,
        "Should have a Type import with 'Bar', got: {:?}",
        imports_to_file_a
    );
}

#[test]
fn inline_type_only_import_does_not_create_circular_dep() {
    let _guard = TEST_MUTEX.lock().unwrap();

    _setup_test();

    let (results, _) = get_results(
        vec![
            SimplifiedFile {
                path: PathBuf::from("./src/index.ts"),
                content: String::from("import { a } from '@src/fileA';"),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileA.ts"),
                content: String::from(
                    r#"
                    import { type TypeB } from '@src/fileB';
                    export const a = 1;
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileB.ts"),
                content: String::from(
                    r#"
                    import { a } from '@src/fileA';
                    export type TypeB = string;
                    "#,
                ),
            },
        ],
        "@src/index.ts",
    );

    let file_a = results.get("./src/fileA.ts").unwrap();
    assert_eq!(
        file_a.circular_deps, None,
        "fileA should not have circular deps: \
         import from fileB is type-only"
    );

    let file_b = results.get("./src/fileB.ts").unwrap();
    assert_eq!(
        file_b.circular_deps, None,
        "fileB should not have circular deps: \
         fileA's import from fileB is type-only"
    );
}

#[test]
fn mixed_inline_type_and_named_import_creates_circular_dep() {
    let _guard = TEST_MUTEX.lock().unwrap();

    _setup_test();

    let (results, _) = get_results(
        vec![
            SimplifiedFile {
                path: PathBuf::from("./src/index.ts"),
                content: String::from("import { a } from '@src/fileA';"),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileA.ts"),
                content: String::from(
                    r#"
                    import { funcB, type TypeB } from '@src/fileB';
                    export const a = 1;
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileB.ts"),
                content: String::from(
                    r#"
                    import { a } from '@src/fileA';
                    export const funcB = 1;
                    export type TypeB = string;
                    "#,
                ),
            },
        ],
        "@src/index.ts",
    );

    let file_a = results.get("./src/fileA.ts").unwrap();
    assert!(
        file_a.circular_deps.is_some(),
        "fileA should have circular deps: \
         funcB is a non-type import from fileB"
    );
}

#[test]
fn inline_type_imports_normalized_separately() {
    let _guard = TEST_MUTEX.lock().unwrap();

    _setup_test();

    let (_, used_files) = get_results(
        vec![
            SimplifiedFile {
                path: PathBuf::from("./src/index.ts"),
                content: String::from(
                    r#"
                    import { foo, type Bar } from '@src/fileA';
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileA.ts"),
                content: String::from(
                    r#"
                    export const foo = 1;
                    export type Bar = string;
                    "#,
                ),
            },
        ],
        "@src/index.ts",
    );

    let index_info = used_files.get("./src/index.ts").unwrap();
    let imports_to_file_a = index_info.imports.get("./src/fileA.ts").unwrap();

    let has_named = imports_to_file_a.iter().any(|i| {
        matches!(
            &i.values,
            ImportType::Named(v) if v.contains(&"foo".to_string())
        )
    });
    let has_type = imports_to_file_a.iter().any(|i| {
        matches!(
            &i.values,
            ImportType::Type(v) if v.contains(&"Bar".to_string())
        )
    });
    let named_does_not_contain_type = imports_to_file_a.iter().all(|i| {
        !matches!(
            &i.values,
            ImportType::Named(v) if v.contains(&"Bar".to_string())
        )
    });

    assert!(
        has_named,
        "Should have Named import with 'foo', got: {:?}",
        imports_to_file_a
    );
    assert!(
        has_type,
        "Should have Type import with 'Bar', got: {:?}",
        imports_to_file_a
    );
    assert!(
        named_does_not_contain_type,
        "'Bar' should be in Type, not Named: {:?}",
        imports_to_file_a
    );
}

// ---- Regression tests for try_merge_import bug ----
// SideEffect/Dynamic imports should NOT be merged with Type imports.
// When merged, the runtime edge is lost and circular deps go undetected.

#[test]
fn side_effect_with_type_import_still_detects_circular_dep() {
    let _guard = TEST_MUTEX.lock().unwrap();
    _setup_test();

    // fileA has a SideEffect import AND a Type import from fileB.
    // The SideEffect creates a runtime edge; the Type should not
    // cause that edge to be dropped.
    let (results, _) = get_results(
        vec![
            SimplifiedFile {
                path: PathBuf::from("./src/index.ts"),
                content: String::from("import { a } from '@src/fileA';"),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileA.ts"),
                content: String::from(
                    r#"
                    import '@src/fileB';
                    import type { Foo } from '@src/fileB';
                    export const a = 1;
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileB.ts"),
                content: String::from(
                    r#"
                    import { a } from '@src/fileA';
                    export type Foo = string;
                    "#,
                ),
            },
        ],
        "@src/index.ts",
    );

    let file_a = results.get("./src/fileA.ts").unwrap();
    assert!(
        file_a.circular_deps.is_some(),
        "fileA should have circular deps: side-effect import \
         from fileB is a runtime dependency"
    );
}

#[test]
fn type_import_then_side_effect_still_detects_circular_dep() {
    let _guard = TEST_MUTEX.lock().unwrap();
    _setup_test();

    // Same scenario but Type import appears before SideEffect
    // in the source. Both orderings must preserve the runtime edge.
    let (results, _) = get_results(
        vec![
            SimplifiedFile {
                path: PathBuf::from("./src/index.ts"),
                content: String::from("import { a } from '@src/fileA';"),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileA.ts"),
                content: String::from(
                    r#"
                    import type { Foo } from '@src/fileB';
                    import '@src/fileB';
                    export const a = 1;
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileB.ts"),
                content: String::from(
                    r#"
                    import { a } from '@src/fileA';
                    export type Foo = string;
                    "#,
                ),
            },
        ],
        "@src/index.ts",
    );

    let file_a = results.get("./src/fileA.ts").unwrap();
    assert!(
        file_a.circular_deps.is_some(),
        "fileA should have circular deps: side-effect import \
         from fileB is a runtime dependency, regardless of \
         order with type import"
    );
}

#[test]
fn dynamic_import_with_type_import_still_detects_circular_dep() {
    let _guard = TEST_MUTEX.lock().unwrap();
    _setup_test();

    // Dynamic import creates a runtime edge that should survive
    // merging with a Type import from the same path.
    let (results, _) = get_results(
        vec![
            SimplifiedFile {
                path: PathBuf::from("./src/index.ts"),
                content: String::from("import { a } from '@src/fileA';"),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileA.ts"),
                content: String::from(
                    r#"
                    const mod = import('@src/fileB');
                    import type { Foo } from '@src/fileB';
                    export const a = 1;
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileB.ts"),
                content: String::from(
                    r#"
                    import { a } from '@src/fileA';
                    export type Foo = string;
                    "#,
                ),
            },
        ],
        "@src/index.ts",
    );

    let file_a = results.get("./src/fileA.ts").unwrap();
    assert!(
        file_a.circular_deps.is_some(),
        "fileA should have circular deps: dynamic import \
         from fileB is a runtime dependency"
    );
}

#[test]
fn type_import_then_dynamic_still_detects_circular_dep() {
    let _guard = TEST_MUTEX.lock().unwrap();
    _setup_test();

    // Type import first, then Dynamic. The Dynamic edge must
    // survive and not be dropped by merge with the existing Type.
    let (results, _) = get_results(
        vec![
            SimplifiedFile {
                path: PathBuf::from("./src/index.ts"),
                content: String::from("import { a } from '@src/fileA';"),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileA.ts"),
                content: String::from(
                    r#"
                    import type { Foo } from '@src/fileB';
                    const mod = import('@src/fileB');
                    export const a = 1;
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileB.ts"),
                content: String::from(
                    r#"
                    import { a } from '@src/fileA';
                    export type Foo = string;
                    "#,
                ),
            },
        ],
        "@src/index.ts",
    );

    let file_a = results.get("./src/fileA.ts").unwrap();
    assert!(
        file_a.circular_deps.is_some(),
        "fileA should have circular deps: dynamic import \
         from fileB is a runtime dependency, regardless of \
         order with type import"
    );
}

// ---- Regression test for direct circular dep display ----
// check_ts_not_have_direct_circular_deps should blame the
// runtime import that causes the cycle, not a type-only import
// that happens to transitively reach the same file.

#[test]
fn direct_circular_dep_error_blames_runtime_import_not_type() {
    let _guard = TEST_MUTEX.lock().unwrap();
    _setup_test();

    // fileA type-imports from fileB, runtime-imports from fileC.
    // fileC imports back from fileA → cycle A→C→A.
    // fileB also reaches fileA via fileC (B→C→A).
    // The error should name fileC, not fileB.
    let (_, _) = get_results(
        vec![
            SimplifiedFile {
                path: PathBuf::from("./src/index.ts"),
                content: String::from("import { a } from '@src/fileA';"),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileA.ts"),
                content: String::from(
                    r#"
                    import type { Foo } from '@src/fileB';
                    import { c } from '@src/fileC';
                    export const a = 1;
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileB.ts"),
                content: String::from(
                    r#"
                    import { c } from '@src/fileC';
                    export type Foo = string;
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/fileC.ts"),
                content: String::from(
                    r#"
                    import { a } from '@src/fileA';
                    export const c = 1;
                    "#,
                ),
            },
        ],
        "@src/index.ts",
    );

    let file_a = File {
        basename: "fileA".to_string(),
        name_with_ext: "fileA.ts".to_string(),
        content: None,
        extension: Some("ts".to_string()),
        relative_path: "./src/fileA.ts".to_string(),
    };

    let result = ts_checks::check_ts_not_have_direct_circular_deps(&file_a);
    let err = result.unwrap_err();

    assert!(
        err.contains("@src/fileC"),
        "Error should blame fileC (the runtime import causing \
         the cycle), not fileB (type-only). Got: {}",
        err
    );
}

#[test]
fn get_resolved_path_consistent_with_non_default_root() {
    let _guard = TEST_MUTEX.lock().unwrap();

    _setup_test();

    *ROOT_DIR.lock().unwrap() = "/project".to_string();
    *ALIASES.lock().unwrap() =
        HashMap::from_iter(vec![(String::from("@src"), String::from("./src"))]);

    let file = File {
        basename: "fileA".to_string(),
        name_with_ext: "fileA.ts".to_string(),
        content: Some("export const a = 1;".to_string()),
        extension: Some("ts".to_string()),
        relative_path: "./src/fileA.ts".to_string(),
    };

    FILES_CACHE
        .lock()
        .unwrap()
        .insert("./src/fileA.ts".to_string(), file);

    let first = get_resolved_path(Path::new("./src/fileA.ts"))
        .unwrap()
        .unwrap();
    let second = get_resolved_path(Path::new("./src/fileA.ts"))
        .unwrap()
        .unwrap();

    assert_eq!(
        first, second,
        "get_resolved_path should return consistent results: \
         first={:?}, second={:?}",
        first, second
    );
    assert_eq!(
        first,
        PathBuf::from("./src/fileA.ts"),
        "Should return relative path, not absolute"
    );
}

#[test]
fn get_resolved_path_extension_consistent_with_non_default_root() {
    let _guard = TEST_MUTEX.lock().unwrap();

    _setup_test();

    *ROOT_DIR.lock().unwrap() = "/project".to_string();
    *ALIASES.lock().unwrap() =
        HashMap::from_iter(vec![(String::from("@src"), String::from("./src"))]);

    let file = File {
        basename: "fileA".to_string(),
        name_with_ext: "fileA.ts".to_string(),
        content: Some("export const a = 1;".to_string()),
        extension: Some("ts".to_string()),
        relative_path: "./src/fileA.ts".to_string(),
    };

    FILES_CACHE
        .lock()
        .unwrap()
        .insert("./src/fileA.ts".to_string(), file);

    // Resolve via alias (goes through extension-trying loop)
    let first = get_resolved_path(Path::new("@src/fileA")).unwrap().unwrap();
    let second = get_resolved_path(Path::new("@src/fileA")).unwrap().unwrap();

    assert_eq!(
        first, second,
        "Extension resolution should be consistent: \
         first={:?}, second={:?}",
        first, second
    );
    assert_eq!(
        first,
        PathBuf::from("./src/fileA.ts"),
        "Should return relative path with extension"
    );
}

#[test]
fn get_resolved_relative_path_for_sibling_import() {
    let _guard = TEST_MUTEX.lock().unwrap();

    _setup_test();

    *ROOT_DIR.lock().unwrap() = "/project".to_string();

    let file = File {
        basename: "fileB".to_string(),
        name_with_ext: "fileB.ts".to_string(),
        content: Some("export const b = 1;".to_string()),
        extension: Some("ts".to_string()),
        relative_path: "./src/feature/fileB.ts".to_string(),
    };

    FILES_CACHE
        .lock()
        .unwrap()
        .insert("./src/feature/fileB.ts".to_string(), file);

    let resolved = get_resolved_path_from(
        Some(Path::new("./src/feature/fileA.ts")),
        Path::new("./fileB.ts"),
    )
    .unwrap()
    .unwrap();

    assert_eq!(resolved, PathBuf::from("./src/feature/fileB.ts"));
}

#[test]
fn get_resolved_relative_path_for_parent_import_with_extension_lookup() {
    let _guard = TEST_MUTEX.lock().unwrap();

    _setup_test();

    *ROOT_DIR.lock().unwrap() = "/project".to_string();

    let file = File {
        basename: "fileB".to_string(),
        name_with_ext: "fileB.ts".to_string(),
        content: Some("export const b = 1;".to_string()),
        extension: Some("ts".to_string()),
        relative_path: "./src/shared/fileB.ts".to_string(),
    };

    FILES_CACHE
        .lock()
        .unwrap()
        .insert("./src/shared/fileB.ts".to_string(), file);

    let resolved = get_resolved_path_from(
        Some(Path::new("./src/feature/fileA.ts")),
        Path::new("../shared/fileB"),
    )
    .unwrap()
    .unwrap();

    assert_eq!(resolved, PathBuf::from("./src/shared/fileB.ts"));
}

#[test]
fn relative_path_resolution_cache_is_scoped_by_importer() {
    let _guard = TEST_MUTEX.lock().unwrap();

    _setup_test();

    *ROOT_DIR.lock().unwrap() = "/project".to_string();

    for relative_path in ["./src/featureA/shared.ts", "./src/featureB/shared.ts"] {
        FILES_CACHE.lock().unwrap().insert(
            relative_path.to_string(),
            File {
                basename: "shared".to_string(),
                name_with_ext: "shared.ts".to_string(),
                content: Some("export const value = 1;".to_string()),
                extension: Some("ts".to_string()),
                relative_path: relative_path.to_string(),
            },
        );
    }

    let first = get_resolved_path_from(
        Some(Path::new("./src/featureA/fileA.ts")),
        Path::new("./shared"),
    )
    .unwrap()
    .unwrap();
    let second = get_resolved_path_from(
        Some(Path::new("./src/featureB/fileB.ts")),
        Path::new("./shared"),
    )
    .unwrap()
    .unwrap();

    assert_eq!(first, PathBuf::from("./src/featureA/shared.ts"));
    assert_eq!(second, PathBuf::from("./src/featureB/shared.ts"));
}

#[test]
fn project_with_relative_and_alias_imports() {
    let _guard = TEST_MUTEX.lock().unwrap();

    _setup_test();

    let (results, _) = get_results(
        vec![
            SimplifiedFile {
                path: PathBuf::from("./src/index.ts"),
                content: String::from("import '@src/feature/fileA';"),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/feature/fileA.ts"),
                content: String::from(
                    r#"
                    import { b } from './fileB';
                    export const a = b;
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/feature/fileB.ts"),
                content: String::from(
                    r#"
                    import { c } from '../shared/fileC';
                    export const b = c;
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/shared/fileC.ts"),
                content: String::from(
                    r#"
                    import { d } from '@src/utils/fileD';
                    export const c = d;
                    "#,
                ),
            },
            SimplifiedFile {
                path: PathBuf::from("./src/utils/fileD.ts"),
                content: String::from("export const d = 1;"),
            },
        ],
        "@src/index.ts",
    );

    assert_eq!(
        results
            .get("./src/index.ts")
            .unwrap()
            .deps
            .iter()
            .cloned()
            .collect::<Vec<_>>(),
        vec![
            "./src/feature/fileA.ts".to_string(),
            "./src/feature/fileB.ts".to_string(),
            "./src/shared/fileC.ts".to_string(),
            "./src/utils/fileD.ts".to_string(),
        ]
    );
}
