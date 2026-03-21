use colored::Colorize;
use globset::Glob;
use std::path::PathBuf;

use crate::{
    analyze_ts_deps::replace_aliases, internal_config::MatchImport,
    load_folder_structure::File, utils::join_and_truncate_string_vec,
};

use super::{
    add_aliases,
    extract_file_content_exports::Export,
    extract_file_content_imports::{Import, ImportType},
    get_file_deps_result, get_file_imports, USED_FILES,
};

pub fn check_ts_not_have_unused_exports(file: &File) -> Result<(), String> {
    let used_files = USED_FILES.lock().unwrap();

    let deps_info = used_files.get(&file.relative_path);

    if let Some(deps_info) = deps_info {
        let mut unused_exports = deps_info
            .exports
            .iter()
            .filter(|export| !export.ignored)
            .cloned()
            .collect::<Vec<_>>();
        let ignored_exports = deps_info
            .exports
            .iter()
            .filter(|export| export.ignored)
            .cloned()
            .collect::<Vec<_>>();
        let mut used_ignored_exports: Vec<Export> = Vec::new();

        let all_ignored_exports_count = ignored_exports.len();

        for (other_used_file, other_deps_info) in used_files.iter() {
            if unused_exports.is_empty() {
                break;
            }

            if other_used_file == &file.relative_path {
                continue;
            }

            if let Some(related_imports) =
                other_deps_info.imports.get(&file.relative_path)
            {
                for related_import in related_imports {
                    match &related_import.values {
                        ImportType::All | ImportType::Dynamic => {
                            unused_exports = vec![];
                            used_ignored_exports
                                .extend(ignored_exports.clone());
                        }
                        ImportType::Named(values) => {
                            unused_exports.retain(|export| {
                                !values.contains(&export.name)
                            });

                            let related_ignored_exports = ignored_exports
                                .iter()
                                .filter(|export| {
                                    values.contains(&export.name)
                                })
                                .cloned()
                                .collect::<Vec<_>>();

                            used_ignored_exports
                                .extend(related_ignored_exports);
                        }
                        ImportType::SideEffect => {}
                        ImportType::Type(values) => {
                            unused_exports.retain(|export| {
                                !values.contains(&export.name)
                            });

                            let related_ignored_exports = ignored_exports
                                .iter()
                                .filter(|export| {
                                    values.contains(&export.name)
                                })
                                .cloned()
                                .collect::<Vec<_>>();

                            used_ignored_exports
                                .extend(related_ignored_exports);
                        }
                        ImportType::Glob => {}
                    }
                }
            }
        }

        let count_of_ignore_next_line_comments = if let Some(content) = &file.content
        {
            content
                .matches("// palinter-ignore-unused-next-line")
                .count()
        } else {
            0
        };

        if count_of_ignore_next_line_comments > all_ignored_exports_count {
            return Err("Unused ignore comment '// palinter-ignore-unused-next-line', remove it".to_string());
        }

        if !used_ignored_exports.is_empty() {
            return Err(format!(
                "Unused ignore comments '// palinter-ignore-unused-next-line', remove them: {}",
                used_ignored_exports
                    .iter()
                    .map(|export| format!(
                        "{} in {}:{}",
                        export.name.blue().bold(),
                        file.relative_path.bright_magenta().bold(),
                        format!("{}", export.line - 1).bright_magenta().bold()
                    ))
                    .collect::<Vec<String>>()
                    .join(" ・ ")
            ));
        }

        if !unused_exports.is_empty() {
            Err(format!(
                "File has unused exports: {}",
                unused_exports
                    .iter()
                    .map(|export| format!(
                        "{} in {}:{}",
                        export.name.blue().bold(),
                        file.relative_path.bright_magenta().bold(),
                        format!("{}", export.line).bright_magenta().bold()
                    ))
                    .collect::<Vec<String>>()
                    .join(" ・ ")
            ))
        } else if file_has_ignore_comment(file, "not-have-unused-exports") {
            Err(
                "Unused ignore comment '// palinter-ignore-not-have-unused-exports', remove it"
                    .to_string(),
            )
        } else {
            Ok(())
        }
    } else if file_has_ignore_comment(file, "not-have-unused-exports") {
        Ok(())
    } else {
        Err("File is not being used in the project".to_string())
    }
}

fn file_has_ignore_comment(file: &File, ignore_comment: &str) -> bool {
    !find_ignore_comment_lines(file, ignore_comment).is_empty()
}

fn find_ignore_comment_lines(
    file: &File,
    ignore_suffix: &str,
) -> Vec<usize> {
    let ignore_comment =
        format!("palinter-ignore-{}", ignore_suffix);
    let line_comment = format!("// {}", ignore_comment);
    let block_comment = format!("/* {}", ignore_comment);

    if let Some(content) = &file.content {
        content
            .lines()
            .enumerate()
            .filter(|(_, line)| {
                let trimmed = line.trim();
                trimmed.contains(&line_comment)
                    || trimmed.contains(&block_comment)
            })
            .map(|(idx, _)| idx + 1) // 1-indexed
            .collect()
    } else {
        vec![]
    }
}

fn import_has_ignore_comment_above(
    file: &File,
    import_line: usize,
    ignore_suffix: &str,
) -> bool {
    if import_line <= 1 {
        return false;
    }

    let ignore_comment =
        format!("palinter-ignore-{}", ignore_suffix);

    if let Some(content) = &file.content {
        let lines: Vec<&str> = content.lines().collect();
        // import_line is 1-indexed, convert to 0-indexed
        // and go one line above
        let above_line_idx = import_line - 2;

        if above_line_idx < lines.len() {
            let above_line = lines[above_line_idx].trim();
            above_line.contains(&format!("// {}", ignore_comment))
                || above_line
                    .contains(&format!("/* {}", ignore_comment))
        } else {
            false
        }
    } else {
        false
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

        if file_has_ignore_comment(file, "not-have-circular-deps") {
            return Ok(());
        }

        Err(format!("File has circular dependencies: {}", circular_deps))
    } else if file_has_ignore_comment(file, "not-have-circular-deps") {
        Err(
            "Unused ignore comment '// palinter-ignore-not-have-circular-deps', remove it"
                .to_string(),
        )
    } else {
        Ok(())
    }
}

pub fn check_ts_not_have_direct_circular_deps(
    file: &File,
) -> Result<(), String> {
    let ignore_suffix = "not-have-direct-circular-deps";
    let deps_info = get_file_deps_result(
        &PathBuf::from(file.clone().relative_path),
    )?;

    let all_comment_lines =
        find_ignore_comment_lines(file, ignore_suffix);

    if deps_info.deps.contains(&file.relative_path) {
        let file_imports = get_file_imports(
            PathBuf::from(file.clone().relative_path)
                .to_str()
                .unwrap(),
        )?;

        let mut used_comment_lines: Vec<usize> = vec![];

        for (resolved_import_path, imports) in &file_imports {
            let has_non_type = imports
                .iter()
                .any(|i| !matches!(i.values, ImportType::Type(_)));
            if !has_non_type {
                continue;
            }

            let import_deps_result = get_file_deps_result(
                &PathBuf::from(resolved_import_path),
            )?;
            if import_deps_result
                .deps
                .contains(&file.relative_path)
            {
                // Check if any non-type import from this path
                // has the ignore comment on the line above
                let ignored_import = imports
                    .iter()
                    .filter(|i| {
                        !matches!(i.values, ImportType::Type(_))
                    })
                    .find(|i| {
                        import_has_ignore_comment_above(
                            file,
                            i.line,
                            ignore_suffix,
                        )
                    });

                if let Some(import) = ignored_import {
                    used_comment_lines.push(import.line - 1);
                    continue;
                }

                let non_type_import = imports
                    .iter()
                    .find(|i| {
                        !matches!(i.values, ImportType::Type(_))
                    });
                let display_path = non_type_import
                    .map(|i| {
                        i.import_path.to_str().unwrap_or("?")
                    })
                    .unwrap_or("?");
                let line = non_type_import
                    .map(|i| i.line)
                    .unwrap_or(0);
                return Err(format!(
                    "File has direct circular dependencies with '{}' in line {} (run cmd `palinter circular-deps [file] -D` to get more info)",
                    display_path, line
                ));
            }
        }

        if used_comment_lines.is_empty() {
            return Err(
                "File has direct circular dependencies (run cmd `palinter circular-deps [file] -D` to get more info)".to_string()
            );
        }

        // All offending imports were ignored — check for
        // excess ignore comments
        let unused_lines: Vec<usize> = all_comment_lines
            .iter()
            .filter(|l| !used_comment_lines.contains(l))
            .copied()
            .collect();

        if !unused_lines.is_empty() {
            return Err(format!(
                "Unused ignore comment '// palinter-ignore-not-have-direct-circular-deps' in line {}, remove it",
                unused_lines.iter().map(|l| l.to_string()).collect::<Vec<_>>().join(", ")
            ));
        }

        Ok(())
    } else if !all_comment_lines.is_empty() {
        Err(format!(
            "Unused ignore comment '// palinter-ignore-not-have-direct-circular-deps' in line {}, remove it",
            all_comment_lines.iter().map(|l| l.to_string()).collect::<Vec<_>>().join(", ")
        ))
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
                if !file_imports
                    .values()
                    .flatten()
                    .any(|Import { import_path, .. }| {
                        match_glob_path(path, import_path)
                    })
                {
                    errors.push(format!(
                        "Should have any import from '{}'",
                        path
                    ));
                }
            }
            MatchImport::DefaultFrom(path) => {
                if !file_imports.values().flatten().any(
                    |Import {
                         import_path,
                         values,
                         ..
                     }| {
                        if match_glob_path(path, import_path) {
                            if let ImportType::Named(values) = values {
                                values
                                    .contains(&"default".to_string())
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
                if !file_imports.values().flatten().any(
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
                if file_imports
                    .values()
                    .flatten()
                    .any(|Import { import_path, .. }| {
                        match_glob_path(path, import_path)
                    })
                {
                    errors.push(format!(
                        "Should not have any import from '{}'",
                        path
                    ));
                }
            }
            MatchImport::DefaultFrom(path) => {
                if file_imports.values().flatten().any(
                    |Import {
                         import_path,
                         values,
                         ..
                     }| {
                        if match_glob_path(path, import_path) {
                            if let ImportType::Named(values) = values {
                                values
                                    .contains(&"default".to_string())
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
                if file_imports.values().flatten().any(
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
