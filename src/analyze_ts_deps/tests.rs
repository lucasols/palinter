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
            path: file.path.to_str().unwrap().to_string(),
        };

        flatten_root_structure
            .insert(file.path.to_str().unwrap().to_string(), file_to_add);
    }

    flatten_root_structure
}

#[test]
fn get_project_files_deps_info_test() {
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
    ]);

    let result = get_used_project_files_deps_info(
        entry_points,
        flattened_root_structure,
        &HashMap::from_iter(vec![(String::from("@src"), String::from("./src"))]),
        &mut TsProjectCtx::default(),
    )
    .unwrap();

    assert_debug_snapshot!(BTreeMap::from_iter(result));
}

#[test]
fn get_project_files_deps_info_test_2() {
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
        &HashMap::from_iter(vec![(String::from("@src"), String::from("./src"))]),
        &mut TsProjectCtx::default(),
    )
    .unwrap();

    assert_debug_snapshot!(BTreeMap::from_iter(result));
}

#[test]
fn project_with_circular_deps() {
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

    let result = get_used_project_files_deps_info(
        entry_points,
        flattened_root_structure,
        &HashMap::from_iter(vec![(String::from("@src"), String::from("./src"))]),
        &mut TsProjectCtx::default(),
    )
    .unwrap();

    assert_debug_snapshot!(BTreeMap::from_iter(result));
}
