use colored::Colorize;
use globset::Glob;
use std::path::PathBuf;

use crate::{analyze_ts_deps::replace_aliases, load_folder_structure::File};

use super::{
    add_aliases, extract_file_content_imports::ImportType, get_file_deps_result,
    USED_FILES,
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
                }
            }
        }

        if !unused_exports.is_empty() {
            return Err(format!(
                "File has unused exports: {}",
                unused_exports
                    .iter()
                    .map(|export| format!(
                        "'{}' in line {}",
                        export.name, export.line
                    ))
                    .collect::<Vec<String>>()
                    .join(", ")
            ));
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
                "disallowed dependencies from folders '{}' found: {}",
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
                "disallowed dependencies outside folders '{}' found: {}",
                allowed.join(", "),
                dep_path.join(" > ")
            ));
        }
    }

    Ok(())
}
