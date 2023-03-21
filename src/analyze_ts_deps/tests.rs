use std::collections::BTreeMap;

use super::*;
use insta::assert_debug_snapshot;

struct SimplifiedFile {
    path: PathBuf,
    content: String,
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
    _setup_test();

    let entry_points = vec![PathBuf::from("@src/index.ts")];

    let flattened_root_structure = create_flatten_root_structure(vec![
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
    ]);

    let ctx = &mut &mut TsProjectCtx {
        root_dir: ".".to_string(),
        ..Default::default()
    };

    let result = get_used_project_files_deps_info(
        entry_points,
        flattened_root_structure,
        HashMap::from_iter(vec![(String::from("@src"), String::from("./src"))]),
        ctx,
    )
    .unwrap();

    assert_debug_snapshot!(BTreeMap::from_iter(result));
}

#[test]
fn get_project_files_deps_info_test_2() {
    _setup_test();

    let entry_points = vec![PathBuf::from("@src/index.ts")];

    let flattened_root_structure = create_flatten_root_structure(vec![
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
    ]);

    let result = get_used_project_files_deps_info(
        entry_points,
        flattened_root_structure,
        HashMap::from_iter(vec![(String::from("@src"), String::from("./src"))]),
        &mut TsProjectCtx::default(),
    )
    .unwrap();

    assert_debug_snapshot!(BTreeMap::from_iter(result));
}

#[test]
fn project_with_circular_deps() {
    _setup_test();

    let entry_points = vec![PathBuf::from("@src/index.ts")];

    let flattened_root_structure = create_flatten_root_structure(vec![
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
    ]);

    let ctx = &mut &mut TsProjectCtx::default();

    let result = get_used_project_files_deps_info(
        entry_points,
        flattened_root_structure,
        HashMap::from_iter(vec![(String::from("@src"), String::from("./src"))]),
        ctx,
    )
    .unwrap();

    // dbg!(&ctx.deps_cache);

    assert_debug_snapshot!(BTreeMap::from_iter(result));
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
    _setup_test();

    let entry_points = vec![PathBuf::from("@src/index.ts")];

    let flattened_root_structure = create_flatten_root_structure(vec![
        get_file("index", &["b", "c"]),
        get_file("c", &["d"]),
        get_file("b", &["d", "e"]),
        get_file("d", &["1"]),
        get_file("e", &["f"]),
        get_file("f", &["b"]),
        get_file("1", &["2"]),
        get_file("2", &["3"]),
        get_file("3", &["1"]),
    ]);

    let ctx = &mut &mut TsProjectCtx {
        root_dir: ".".to_string(),
        ..Default::default()
    };

    let result = get_used_project_files_deps_info(
        entry_points,
        flattened_root_structure,
        HashMap::from_iter(vec![(String::from("@src"), String::from("./src"))]),
        ctx,
    )
    .unwrap();

    assert_debug_snapshot!(BTreeMap::from_iter(result));
}

#[test]
fn project_with_circular_deps_3() {
    _setup_test();

    let entry_points = vec![PathBuf::from("@src/a.ts")];

    let flattened_root_structure = create_flatten_root_structure(vec![
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
    ]);

    let ctx = &mut &mut TsProjectCtx {
        root_dir: ".".to_string(),
        ..Default::default()
    };

    let result = get_used_project_files_deps_info(
        entry_points,
        flattened_root_structure,
        HashMap::from_iter(vec![(String::from("@src"), String::from("./src"))]),
        ctx,
    )
    .unwrap();

    assert_debug_snapshot!(BTreeMap::from_iter(result));
}
