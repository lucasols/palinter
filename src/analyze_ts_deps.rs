use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fs::read_to_string,
    path::{Path, PathBuf},
};

use crate::{
    internal_config::Config,
    load_folder_structure::{get_flattened_files_structure, File, Folder},
    utils::clone_extend_vec,
};

use self::{
    extract_file_content_exports::{extract_file_content_exports, Export},
    extract_file_content_imports::{
        extract_imports_from_file_content, Import, ImportType,
    },
};
mod extract_file_content_exports;
mod extract_file_content_imports;
pub mod ts_checks;

#[derive(Debug)]
pub struct FileDepsInfo {
    deps: BTreeSet<PathBuf>,
    has_circular_deps: bool,
    imports: BTreeMap<String, Import>,
    exports: Vec<Export>,
}

fn load_file_from_cache(
    file_path: &PathBuf,
    files_cache: &mut HashMap<String, File>,
) -> Result<File, String> {
    let from_cache = files_cache.get(file_path.to_str().unwrap());

    let related_file = match from_cache {
        Some(file) => file.clone(),
        None => {
            let new_file = load_file_from_path(file_path)?;

            files_cache
                .insert(file_path.to_str().unwrap().to_string(), new_file.clone());

            new_file
        }
    };

    Ok(related_file)
}

fn get_resolved_path(
    path: &Path,
    aliases: &HashMap<String, String>,
    files_cache: &mut HashMap<String, File>,
    resolve_cache: &mut HashMap<PathBuf, PathBuf>,
) -> Result<Option<PathBuf>, String> {
    if !aliases
        .iter()
        .any(|(alias, replace)| path.starts_with(alias) || path.starts_with(replace))
    {
        return Ok(None);
    }

    if let Some(resolved_path) = resolve_cache.get(path) {
        return Ok(Some(resolved_path.clone()));
    }

    let file_with_replaced_alias = replace_aliases(aliases, path);

    let file_extension = file_with_replaced_alias.extension();

    if file_extension.is_none() {
        let file_name_start_with_uppercase = file_with_replaced_alias
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .chars()
            .next()
            .unwrap()
            .is_uppercase();

        let test_extensions = if file_name_start_with_uppercase {
            vec!["tsx", "ts"]
        } else {
            vec!["ts", "tsx"]
        };

        for ext_to_try in &test_extensions {
            let new_path = file_with_replaced_alias.with_extension(ext_to_try);

            let new_file = load_file_from_cache(&new_path, files_cache);

            if new_file.is_ok() {
                resolve_cache.insert(path.to_path_buf(), new_path.clone());

                return Ok(Some(new_path));
            }
        }

        return Err(format!(
            "TS: Can't find file with extensions .ts or .tsx for path: {:?}",
            file_with_replaced_alias
        ));
    } else {
        resolve_cache.insert(path.to_path_buf(), file_with_replaced_alias.clone());

        Ok(Some(file_with_replaced_alias))
    }
}

fn replace_aliases(aliases: &HashMap<String, String>, path: &Path) -> PathBuf {
    for (alias, real_path) in aliases {
        if path.starts_with(alias) {
            return PathBuf::from(path.to_str().unwrap().replace(alias, real_path));
        }
    }

    path.to_path_buf()
}

fn visit_file(
    resolved_path: &PathBuf,
    files_cache: &mut HashMap<String, File>,
    result: &mut HashMap<String, FileDepsInfo>,
    aliases: &HashMap<String, String>,
    visited: &mut HashSet<PathBuf>,
    path: &mut HashSet<PathBuf>,
    resolve_cache: &mut HashMap<PathBuf, PathBuf>,
) -> Result<(), String> {
    if visited.contains(resolved_path) {
        return Ok(());
    }

    visited.insert(resolved_path.clone());
    path.insert(resolved_path.clone());

    let related_file = load_file_from_cache(resolved_path, files_cache)?;

    let file_content = related_file.content.as_ref().ok_or(format!(
        "TS: Error getting file content of: {}, check if the file type is added to the config to be analyzed",
        resolved_path.to_str().unwrap_or("invalid path")
    ))?;

    let file_imports = extract_imports_from_file_content(file_content)?;

    let mut deps: BTreeSet<PathBuf> = BTreeSet::new();

    for import in &file_imports {
        if let Some(edge_resolved_path) = get_resolved_path(
            &import.import_path,
            aliases,
            files_cache,
            resolve_cache,
        )? {
            visit_file(
                &edge_resolved_path,
                files_cache,
                result,
                aliases,
                visited,
                path,
                resolve_cache,
            )?;

            if let Some(edge_info) =
                result.get_mut(edge_resolved_path.to_str().unwrap())
            {
                deps.extend(edge_info.deps.clone());
            }

            deps.insert(edge_resolved_path.clone());
        }
    }

    let file_exports = extract_file_content_exports(file_content)?;

    result.insert(
        resolved_path.to_str().unwrap().to_string(),
        FileDepsInfo {
            has_circular_deps: deps.contains(resolved_path),
            deps,
            imports: normalize_imports(
                file_imports,
                aliases,
                files_cache,
                resolve_cache,
            )?,
            exports: file_exports,
        },
    );

    path.remove(resolved_path);

    Ok(())
}

fn normalize_imports(
    imports: Vec<Import>,
    aliases: &HashMap<String, String>,
    files_cache: &mut HashMap<String, File>,
    resolve_cache: &mut HashMap<PathBuf, PathBuf>,
) -> Result<BTreeMap<String, Import>, String> {
    let mut normalized_imports: BTreeMap<String, Import> = BTreeMap::new();

    for import in imports {
        let resolved_import_name = get_resolved_path(
            &import.import_path,
            aliases,
            files_cache,
            resolve_cache,
        )?;

        let use_name =
            pb_to_string(resolved_import_name.unwrap_or(import.import_path.clone()));

        let current_import = normalized_imports.get(&use_name);

        if let Some(current_import) = current_import {
            match &current_import.values {
                ImportType::All => {
                    continue;
                }
                ImportType::Named(current_named) => {
                    let new_values = match &import.values {
                        ImportType::All => ImportType::All,
                        ImportType::Named(new_named) => ImportType::Named(
                            clone_extend_vec(current_named, new_named),
                        ),
                        ImportType::Dynamic | ImportType::SideEffect => {
                            ImportType::Named(current_named.clone())
                        }
                    };

                    normalized_imports.insert(
                        use_name,
                        Import {
                            import_path: import.import_path,
                            line: import.line,
                            values: new_values,
                        },
                    );
                }
                ImportType::SideEffect | ImportType::Dynamic => {
                    normalized_imports.insert(use_name, import);
                }
            }
        } else {
            normalized_imports.insert(use_name, import);
        }
    }

    Ok(normalized_imports)
}

fn pb_to_string(import_path: PathBuf) -> String {
    import_path.to_str().unwrap().to_string()
}

pub fn load_file_from_path(path: &PathBuf) -> Result<File, String> {
    let file_content = match read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            return Err(format!(
                "TS: Error reading file: {}, Error: {}",
                path.to_str().unwrap_or("invalid path"),
                err
            ))
        }
    };

    let file = File {
        basename: path.file_stem().unwrap().to_str().unwrap().to_string(),
        name_with_ext: path.file_name().unwrap().to_str().unwrap().to_string(),
        content: Some(file_content),
        extension: path
            .extension()
            .map(|ext| ext.to_str().unwrap().to_string()),
        path: path.to_str().unwrap().to_string(),
    };

    Ok(file)
}

fn get_used_project_files_deps_info(
    entry_points: Vec<PathBuf>,
    flattened_root_structure: HashMap<String, File>,
    aliases: &HashMap<String, String>,
) -> Result<HashMap<String, FileDepsInfo>, String> {
    let mut result: HashMap<String, FileDepsInfo> = HashMap::new();

    let mut files_cache = flattened_root_structure;
    let mut resolve_cache = HashMap::new();

    for entry in entry_points {
        let mut visited_files = HashSet::new();
        let mut path = HashSet::new();

        if let Some(resolved_path) =
            get_resolved_path(&entry, aliases, &mut files_cache, &mut resolve_cache)?
        {
            visit_file(
                &resolved_path,
                &mut files_cache,
                &mut result,
                aliases,
                &mut visited_files,
                &mut path,
                &mut resolve_cache,
            )?;
        }
    }

    Ok(result)
}

pub fn get_used_project_files_deps_info_from_cfg(
    config: &Config,
    root_structure: &Folder,
) -> Result<HashMap<String, FileDepsInfo>, String> {
    let unused_exports_entry_points = config
        .clone()
        .ts_config
        .map(|c| {
            c.unused_exports_entry_points
                .iter()
                .map(PathBuf::from)
                .collect::<Vec<PathBuf>>()
        })
        .unwrap_or_default();

    if unused_exports_entry_points.is_empty() {
        return Ok(HashMap::default());
    }

    let flattened_root_structure = if !unused_exports_entry_points.is_empty() {
        get_flattened_files_structure(root_structure)
    } else {
        HashMap::default()
    };

    get_used_project_files_deps_info(
        unused_exports_entry_points,
        flattened_root_structure,
        &config
            .ts_config
            .as_ref()
            .map(|c| c.aliases.clone())
            .unwrap_or_default(),
    )
}

#[cfg(test)]
mod tests;
