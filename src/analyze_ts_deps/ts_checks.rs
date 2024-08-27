use colored::Colorize;
use globset::Glob;
use std::path::PathBuf;

use crate::{
    analyze_ts_deps::replace_aliases, internal_config::MatchImport,
    load_folder_structure::File, utils::join_and_truncate_string_vec,
};

use super::{
    add_aliases,
    extract_file_content_imports::{Import, ImportType},
    get_file_deps_result, get_file_imports, USED_FILES,
};

pub fn check_ts_not_have_unused_exports(file: &File) -> Result<(), String> {
    let used_files = USED_FILES.lock().unwrap();

    let deps_info = used_files.get(&file.relative_path);

    if let Some(deps_info) = deps_info {
        let mut unused_exports = deps_info.exports.clone();

        for (other_used_file, other_deps_info) in used_files.iter() {
            if unused_exports.is_empty() {
                break;
            }

            if other_used_file == &file.relative_path {
                continue;
            }

            if let Some(related_import) =
                other_deps_info.imports.get(&file.relative_path)
            {
                match &related_import.values {
                    ImportType::All | ImportType::Dynamic => {
                        unused_exports = vec![];
                    }
                    ImportType::Named(values) => {
                        unused_exports
                            .retain(|export| !values.contains(&export.name));
                    }
                    ImportType::SideEffect => {}
                    ImportType::Type(values) => {
                        unused_exports
                            .retain(|export| !values.contains(&export.name));
                    }
                    ImportType::Glob => {}
                }
            }
        }

        if !unused_exports.is_empty() {
            Err(format!(
                "File has unused exports: {}",
                unused_exports
                    .iter()
                    .map(|export| format!(
                        "'{}' in {}:{}",
                        export.name, file.relative_path, export.line
                    ))
                    .collect::<Vec<String>>()
                    .join(" , ")
            ))
        } else {
            Ok(())
        }
    } else {
        Err("File is not being used in the project".to_string())
    }
}

pub fn check_ts_not_have_circular_deps(file: &File) -> Result<(), String> {
    let deps_info =
        get_file_deps_result(&PathBuf::from(file.clone().relative_path))?;

    if let Some(circular_deps) = &deps_info.circular_deps {
        let mut circular_deps = circular_deps.join(", ");

        let original_len = circular_deps.len();

        circular_deps.truncate(100);

        if original_len > 200 {
            circular_deps.push_str("...");
        }

        circular_deps.push_str(
            &" (run cmd `palinter circular-deps [file]` to get more info)".dimmed(),
        );

        Err(format!("File has circular dependencies: {}", circular_deps))
    } else {
        Ok(())
    }
}

pub fn check_ts_not_have_direct_circular_deps(file: &File) -> Result<(), String> {
    let deps_info =
        get_file_deps_result(&PathBuf::from(file.clone().relative_path))?;

    if deps_info.deps.contains(&file.relative_path) {
        Err("File has direct circular dependencies (run cmd `palinter circular-deps [file]` to get more info)".to_string())
    } else {
        Ok(())
    }
}

pub fn check_ts_not_have_deps_from(
    file: &File,
    disallow: &[String],
) -> Result<(), String> {
    let deps_info =
        get_file_deps_result(&PathBuf::from(file.clone().relative_path))?;

    let mut builder = globset::GlobSetBuilder::new();

    for pattern in disallow {
        builder.add(Glob::new(replace_aliases(pattern).as_str()).unwrap());
    }

    let disable_imports_set = builder.build().unwrap();

    let mut dep_path: Vec<String> = vec![];

    for dep in &deps_info.deps {
        dep_path.push(add_aliases(dep));

        if disable_imports_set.is_match(dep) {
            return Err(format!(
                "disallowed dependencies from '{}' found: {}",
                disallow.join(", "),
                dep_path.join(" > ")
            ));
        }
    }

    Ok(())
}

pub fn check_ts_not_have_deps_outside(
    file: &File,
    allowed: &[String],
) -> Result<(), String> {
    let deps_info =
        get_file_deps_result(&PathBuf::from(file.clone().relative_path))?;

    let mut builder = globset::GlobSetBuilder::new();

    for pattern in allowed {
        builder.add(Glob::new(replace_aliases(pattern).as_str()).unwrap());
    }

    let allowed_imports_set = builder.build().unwrap();

    let mut dep_path: Vec<String> = vec![];

    for dep in &deps_info.deps {
        dep_path.push(add_aliases(dep));

        if !allowed_imports_set.is_match(dep) {
            return Err(format!(
                "disallowed dependencies outside '{}' found: {}",
                allowed.join(", "),
                dep_path.join(" > ")
            ));
        }
    }

    Ok(())
}

pub fn check_ts_not_have_used_exports_outside(
    file: &File,
    allowed: &[String],
) -> Result<(), String> {
    let used_files = USED_FILES.lock().unwrap();

    let mut builder = globset::GlobSetBuilder::new();

    for pattern in allowed {
        builder.add(Glob::new(replace_aliases(pattern).as_str()).unwrap());
    }

    let allowed_to_use_exports_set = builder.build().unwrap();

    let mut errors = vec![];

    for (other_used_file, other_deps_info) in used_files.iter() {
        if other_used_file == &file.relative_path {
            continue;
        }

        if other_deps_info.imports.contains_key(&file.relative_path)
            && !allowed_to_use_exports_set.is_match(other_used_file)
        {
            errors.push(add_aliases(other_used_file));
        }
    }

    errors.sort();

    if !errors.is_empty() {
        Err(format!(
            "disallowed used exports in files '{}', this file can only be imported from '{}'",
            join_and_truncate_string_vec(&errors, ", ", 3),
            allowed.join(", ")
        ))
    } else {
        Ok(())
    }
}

pub fn check_ts_have_imports(
    file: &File,
    have_imports: &Vec<MatchImport>,
) -> Result<(), String> {
    let file_imports = get_file_imports(
        PathBuf::from(file.clone().relative_path).to_str().unwrap(),
    )?;

    let mut errors: Vec<String> = vec![];

    for have_import in have_imports {
        match have_import {
            MatchImport::From(path) => {
                if !file_imports.values().any(|Import { import_path, .. }| {
                    match_glob_path(path, import_path)
                }) {
                    errors.push(format!("Should have any import from '{}'", path));
                }
            }
            MatchImport::DefaultFrom(path) => {
                if !file_imports.values().any(
                    |Import {
                         import_path,
                         values,
                         ..
                     }| {
                        if match_glob_path(path, import_path) {
                            if let ImportType::Named(values) = values {
                                values.contains(&"default".to_string())
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    },
                ) {
                    errors.push(format!(
                        "Should have a default import from '{}'",
                        path
                    ));
                }
            }
            MatchImport::Named { from, name } => {
                if !file_imports.values().any(
                    |Import {
                         import_path,
                         values,
                         ..
                     }| {
                        if match_glob_path(from, import_path) {
                            if let ImportType::Named(values) = values {
                                values.contains(name)
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    },
                ) {
                    errors.push(format!(
                        "Should have a named import '{}' from '{}'",
                        name, from
                    ));
                }
            }
        }
    }

    if !errors.is_empty() {
        Err(errors.join(", "))
    } else {
        Ok(())
    }
}

fn match_glob_path(path: &String, import_path: &PathBuf) -> bool {
    globset::Glob::new(replace_aliases(path).as_str())
        .unwrap()
        .compile_matcher()
        .is_match(import_path)
}

pub fn check_ts_not_have_imports(
    file: &File,
    not_have_imports: &Vec<MatchImport>,
) -> Result<(), String> {
    let file_imports = get_file_imports(
        PathBuf::from(file.clone().relative_path).to_str().unwrap(),
    )?;

    let mut errors: Vec<String> = vec![];

    for not_have_import in not_have_imports {
        match not_have_import {
            MatchImport::From(path) => {
                if file_imports.values().any(|Import { import_path, .. }| {
                    match_glob_path(path, import_path)
                }) {
                    errors
                        .push(format!("Should not have any import from '{}'", path));
                }
            }
            MatchImport::DefaultFrom(path) => {
                if file_imports.values().any(
                    |Import {
                         import_path,
                         values,
                         ..
                     }| {
                        if match_glob_path(path, import_path) {
                            if let ImportType::Named(values) = values {
                                values.contains(&"default".to_string())
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    },
                ) {
                    errors.push(format!(
                        "Should not have a default import from '{}'",
                        path
                    ));
                }
            }
            MatchImport::Named { from, name } => {
                if file_imports.values().any(
                    |Import {
                         import_path,
                         values,
                         ..
                     }| {
                        if match_glob_path(from, import_path) {
                            if let ImportType::Named(values) = values {
                                values.contains(name)
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    },
                ) {
                    errors.push(format!(
                        "Should not have a named import '{}' from '{}'",
                        name, from
                    ));
                }
            }
        }
    }

    if !errors.is_empty() {
        Err(errors.join(", "))
    } else {
        Ok(())
    }
}
