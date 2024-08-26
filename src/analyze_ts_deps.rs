use std::{
    collections::HashMap,
    fs::read_to_string,
    path::{Path, PathBuf},
    sync::Mutex,
};

use indexmap::IndexMap;
use lazy_static::lazy_static;

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
    modules_graph::{get_node_deps, DepsResult, DEPS_CACHE},
};
pub mod circular_deps;
mod extract_file_content_exports;
mod extract_file_content_imports;
mod modules_graph;
pub mod ts_checks;

#[derive(Debug, Clone, Default)]
pub struct FileDepsInfo {
    imports: IndexMap<String, Import>,
    exports: Vec<Export>,
}

#[derive(Debug, Clone, Default)]
struct FileEdgesCache {
    edges_without_types: Vec<String>,
    edges: Vec<String>,
}

lazy_static! {
    static ref FILES_CACHE: Mutex<HashMap<String, File>> =
        Mutex::new(HashMap::new());
    static ref RESOLVE_CACHE: Mutex<HashMap<PathBuf, PathBuf>> =
        Mutex::new(HashMap::new());
    static ref IMPORTS_CACHE: Mutex<HashMap<String, IndexMap<String, Import>>> =
        Mutex::new(HashMap::new());
    pub static ref ALIASES: Mutex<HashMap<String, String>> =
        Mutex::new(HashMap::new());
    static ref ROOT_DIR: Mutex<String> = Mutex::new(String::from("."));
    static ref DEBUG_READ_EDGES_COUNT: Mutex<usize> = Mutex::new(0);
    static ref FILE_EDGES_CACHE: Mutex<HashMap<String, FileEdgesCache>> =
        Mutex::new(HashMap::new());
    pub static ref USED_FILES: Mutex<HashMap<String, FileDepsInfo>> =
        Mutex::new(HashMap::new());
    static ref FILE_DEPS_RESULT_CACHE: Mutex<HashMap<String, DepsResult>> =
        Mutex::new(HashMap::new());
}

pub fn _setup_test() {
    DEPS_CACHE.lock().unwrap().clear();
    FILES_CACHE.lock().unwrap().clear();
    RESOLVE_CACHE.lock().unwrap().clear();
    IMPORTS_CACHE.lock().unwrap().clear();
    ALIASES.lock().unwrap().clear();
    *ROOT_DIR.lock().unwrap() = String::from(".");
    *DEBUG_READ_EDGES_COUNT.lock().unwrap() = 0;
    FILE_EDGES_CACHE.lock().unwrap().clear();
    USED_FILES.lock().unwrap().clear();
    FILE_DEPS_RESULT_CACHE.lock().unwrap().clear();
}

fn load_file_from_cache(file_path: &PathBuf) -> Result<File, String> {
    let mut from_cache_binding = FILES_CACHE.lock().unwrap();

    let from_cache = from_cache_binding.get(file_path.to_str().unwrap());

    let related_file = match from_cache {
        Some(file) => file.clone(),
        None => {
            let new_file = load_file_from_path(file_path)?;

            from_cache_binding
                .insert(file_path.to_str().unwrap().to_string(), new_file.clone());

            new_file
        }
    };

    Ok(related_file)
}

fn get_file_deps_result(file_path: &Path) -> Result<DepsResult, String> {
    let mut binding = FILE_DEPS_RESULT_CACHE.lock().unwrap();

    let from_cache = binding.get(file_path.to_str().unwrap());

    match from_cache {
        Some(file) => Ok(file.clone()),
        None => {
            let deps = get_node_deps(
                &file_path.to_str().unwrap().to_string(),
                &mut |path| get_file_edges(path, true),
                None,
                false,
                false,
            )?;

            binding.insert(file_path.to_str().unwrap().to_string(), deps.clone());

            Ok(deps)
        }
    }
}

fn get_resolved_path(path: &Path) -> Result<Option<PathBuf>, String> {
    if !ALIASES
        .lock()
        .unwrap()
        .iter()
        .any(|(alias, replace)| path.starts_with(alias) || path.starts_with(replace))
    {
        return Ok(None);
    }

    if let Some(resolved_path) = RESOLVE_CACHE.lock().unwrap().get(path) {
        return Ok(Some(resolved_path.clone()));
    }

    let file_with_replaced_alias =
        PathBuf::from(replace_aliases(&path.to_str().unwrap().to_string()));

    let file_abs_path = format!(
        "{}{}",
        ROOT_DIR.lock().unwrap(),
        file_with_replaced_alias
            .to_str()
            .unwrap()
            .trim_start_matches('.')
    );

    let file_exists = FILES_CACHE
        .lock()
        .unwrap()
        .contains_key(file_with_replaced_alias.to_str().unwrap())
        || PathBuf::from(file_abs_path.clone()).is_file();

    if !file_exists {
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
            vec![".tsx", ".ts", "/index.tsx", "/index.ts"]
        } else {
            vec![".ts", ".tsx", "/index.ts", "/index.tsx"]
        };

        for paths_to_try in &test_extensions {
            let cache_name = format!(
                "{}{}",
                file_with_replaced_alias.to_str().unwrap(),
                paths_to_try
            );

            let file_is_in_cache =
                FILES_CACHE.lock().unwrap().contains_key(&cache_name);

            let new_path = if file_is_in_cache {
                PathBuf::from(cache_name)
            } else {
                PathBuf::from(format!("{}{}", file_abs_path, paths_to_try))
            };

            let new_file = load_file_from_cache(&new_path);

            if new_file.is_ok() {
                RESOLVE_CACHE
                    .lock()
                    .unwrap()
                    .insert(path.to_path_buf(), new_path.clone());

                return Ok(Some(new_path));
            }
        }

        Err(format!(
            "TS: Can't resolve path: {:?}",
            file_with_replaced_alias
        ))
    } else {
        RESOLVE_CACHE
            .lock()
            .unwrap()
            .insert(path.to_path_buf(), PathBuf::from(file_abs_path));

        Ok(Some(file_with_replaced_alias))
    }
}

fn replace_aliases(path: &String) -> String {
    for (alias, real_path) in ALIASES.lock().unwrap().iter() {
        if path.starts_with(alias) {
            return path.replace(alias, real_path);
        }
    }

    path.to_string()
}

fn add_aliases(path: &String) -> String {
    for (alias, real_path) in ALIASES.lock().unwrap().iter() {
        if path.starts_with(real_path) {
            return path.replace(real_path, alias);
        }
    }

    path.to_string()
}

pub fn get_file_imports(
    resolved_path: &str,
) -> Result<IndexMap<String, Import>, String> {
    let mut binding = IMPORTS_CACHE.lock().unwrap();

    let from_cache = binding.get(resolved_path);

    if let Some(imports) = from_cache {
        return Ok(imports.clone());
    }

    if let Some(file_content) = get_file_content(resolved_path)? {
        let edges_imports =
            normalize_imports(extract_imports_from_file_content(&file_content)?)?;

        binding.insert(resolved_path.to_string(), edges_imports.clone());

        Ok(edges_imports)
    } else {
        Ok(IndexMap::new())
    }
}

fn get_file_edges(
    unresolved_path: &str,
    ignore_type_imports: bool,
) -> Result<Vec<String>, String> {
    let resolved_path = get_resolved_path(Path::new(unresolved_path))?;

    let mut file_edges_cache = FILE_EDGES_CACHE.lock().unwrap();

    if *DEBUG_READ_EDGES_COUNT.lock().unwrap() > 2_000_000 {
        panic!("Too many edges read, probably infinite loop");
    }

    *DEBUG_READ_EDGES_COUNT.lock().unwrap() += 1;

    if let Some(cached_edges) = file_edges_cache.get(unresolved_path) {
        return Ok(if ignore_type_imports {
            cached_edges.edges_without_types.clone()
        } else {
            cached_edges.edges.clone()
        });
    }

    if let Some(resolved_path) = resolved_path {
        if let Some(resolved_path_ext) =
            resolved_path.extension().and_then(|s| s.to_str())
        {
            if resolved_path_ext != "ts" && resolved_path_ext != "tsx" {
                return Ok(vec![]);
            }
        }

        let edges_imports = get_file_imports(resolved_path.to_str().unwrap())?;

        let mut non_type_edges: Vec<String> = Vec::new();
        let mut edges: Vec<String> = Vec::new();

        for import in edges_imports.values() {
            if let Some(resolved_path) = get_resolved_path(&import.import_path)? {
                let path_str = resolved_path.to_str().unwrap().to_string();

                match import.values {
                    ImportType::Type(_) => {
                        edges.push(path_str);
                    }
                    _ => {
                        non_type_edges.push(path_str.clone());
                        edges.push(path_str);
                    }
                }
            }
        }

        file_edges_cache.insert(
            unresolved_path.to_string(),
            FileEdgesCache {
                edges: edges.clone(),
                edges_without_types: non_type_edges.clone(),
            },
        );

        Ok(if ignore_type_imports {
            non_type_edges
        } else {
            edges
        })
    } else {
        file_edges_cache
            .insert(unresolved_path.to_string(), FileEdgesCache::default());

        Ok(vec![])
    }
}

fn get_file_content(resolved_path: &str) -> Result<Option<String>, String> {
    let related_file = load_file_from_cache(&PathBuf::from(resolved_path))?;

    Ok(related_file.content)
}

fn visit_file(
    resolved_path: &Path,
    result: &mut HashMap<String, FileDepsInfo>,
) -> Result<(), String> {
    let resolved_path_string = resolved_path.to_str().unwrap().to_string();

    let file_deps_info = get_basic_file_deps_info(&resolved_path_string)?;

    result.insert(resolved_path_string.clone(), file_deps_info);

    let edges = get_file_edges(&resolved_path_string, false)?;

    for edge in edges {
        let edge_path = PathBuf::from(edge.clone());

        if !result.contains_key(&edge) {
            visit_file(&edge_path, result)?;
        }
    }

    Ok(())
}

pub fn get_basic_file_deps_info(
    resolved_path_string: &str,
) -> Result<FileDepsInfo, String> {
    let file_content = get_file_content(resolved_path_string)?;

    if let Some(file_content) = file_content {
        let exports = extract_file_content_exports(&file_content)?;

        let imports = get_file_imports(resolved_path_string)?;

        let file_deps_info = FileDepsInfo { exports, imports };

        Ok(file_deps_info)
    } else {
        Ok(FileDepsInfo::default())
    }
}

fn normalize_imports(
    imports: Vec<Import>,
) -> Result<IndexMap<String, Import>, String> {
    let mut normalized_imports: IndexMap<String, Import> = IndexMap::new();

    for new_import in imports {
        let resolved_import_name = if new_import.values == ImportType::Glob {
            None
        } else {
            get_resolved_path(&new_import.import_path)?
        };

        let use_name = pb_to_string(
            resolved_import_name.unwrap_or(new_import.import_path.clone()),
        );

        let current_import = normalized_imports.get(&use_name);

        if let Some(current_import) = current_import {
            match &current_import.values {
                ImportType::All | ImportType::Glob => {
                    continue;
                }
                ImportType::Named(current_named) => {
                    let new_values = match &new_import.values {
                        ImportType::All => ImportType::All,
                        ImportType::Named(new_named)
                        | ImportType::Type(new_named) => ImportType::Named(
                            clone_extend_vec(current_named, new_named),
                        ),
                        ImportType::Dynamic
                        | ImportType::SideEffect
                        | ImportType::Glob => {
                            ImportType::Named(current_named.clone())
                        }
                    };

                    normalized_imports.insert(
                        use_name,
                        Import {
                            import_path: new_import.import_path,
                            line: new_import.line,
                            values: new_values,
                        },
                    );
                }
                ImportType::Type(current_types) => {
                    let new_values = match &new_import.values {
                        ImportType::All => ImportType::All,
                        ImportType::Named(new_named) => ImportType::Named(
                            clone_extend_vec(current_types, new_named),
                        ),
                        ImportType::Type(new_types) => ImportType::Type(
                            clone_extend_vec(current_types, new_types),
                        ),
                        ImportType::Dynamic
                        | ImportType::SideEffect
                        | ImportType::Glob => {
                            ImportType::Named(current_types.clone())
                        }
                    };

                    normalized_imports.insert(
                        use_name,
                        Import {
                            import_path: new_import.import_path,
                            line: new_import.line,
                            values: new_values,
                        },
                    );
                }
                ImportType::SideEffect | ImportType::Dynamic => {
                    normalized_imports.insert(use_name, new_import);
                }
            }
        } else {
            match &new_import.values {
                ImportType::Glob => {
                    let normalized_glob = if new_import.import_path.starts_with("/")
                    {
                        format!(".{}", new_import.import_path.to_str().unwrap())
                    } else {
                        new_import.import_path.to_str().unwrap().to_string()
                    };

                    let glob = globset::Glob::new(&normalized_glob)
                        .unwrap()
                        .compile_matcher();

                    for file in FILES_CACHE.lock().unwrap().values() {
                        if glob.is_match(file.relative_path.as_str()) {
                            normalized_imports.insert(
                                file.relative_path.clone(),
                                Import {
                                    import_path: PathBuf::from(
                                        file.relative_path.clone(),
                                    ),
                                    line: new_import.line,
                                    values: ImportType::All,
                                },
                            );
                        }
                    }
                }
                _ => {
                    normalized_imports.insert(use_name, new_import);
                }
            }
        }
    }

    Ok(normalized_imports)
}

fn pb_to_string(import_path: PathBuf) -> String {
    import_path.to_str().unwrap().to_string()
}

pub fn load_file_from_path(path: &PathBuf) -> Result<File, String> {
    let file_content = if path
        .extension()
        .map(|ext| ext == "ts" || ext == "tsx")
        .unwrap_or(false)
    {
        match read_to_string(path) {
            Ok(content) => Some(content),
            Err(err) => {
                return Err(format!(
                    "TS: Error reading file: {}, Error: {}",
                    path.to_str().unwrap_or("invalid path"),
                    err
                ))
            }
        }
    } else {
        None
    };

    let file = File {
        basename: path.file_stem().unwrap().to_str().unwrap().to_string(),
        name_with_ext: path.file_name().unwrap().to_str().unwrap().to_string(),
        content: file_content,
        extension: path
            .extension()
            .map(|ext| ext.to_str().unwrap().to_string()),
        relative_path: path.to_str().unwrap().to_string(),
    };

    Ok(file)
}

fn get_used_project_files_deps_info(
    entry_points: Vec<PathBuf>,
    flattened_root_structure: HashMap<String, File>,
    aliases: HashMap<String, String>,
) -> Result<(), String> {
    let mut result: HashMap<String, FileDepsInfo> = HashMap::new();

    FILES_CACHE.lock().unwrap().extend(flattened_root_structure);

    *ALIASES.lock().unwrap() = aliases;

    for entry in entry_points {
        if let Some(resolved_path) = get_resolved_path(&entry)? {
            visit_file(&resolved_path, &mut result)?;
        }
    }

    *USED_FILES.lock().unwrap() = result;

    Ok(())
}

pub fn load_used_project_files_deps_info_from_cfg(
    config: &Config,
    root_structure: &Folder,
    root_path: &Path,
) -> Result<(), String> {
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
        return Ok(());
    }

    *ROOT_DIR.lock().unwrap() = root_path.to_str().unwrap().to_string();

    let flattened_root_structure = if !unused_exports_entry_points.is_empty() {
        get_flattened_files_structure(root_structure)
    } else {
        HashMap::default()
    };

    get_used_project_files_deps_info(
        unused_exports_entry_points,
        flattened_root_structure,
        config
            .ts_config
            .as_ref()
            .map(|c| c.aliases.clone())
            .unwrap_or_default(),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests;
