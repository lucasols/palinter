use std::{
    collections::{HashMap, HashSet},
    fs::read_to_string,
    path::{Component, Path, PathBuf},
    sync::Mutex,
};

use indexmap::{IndexMap, IndexSet};
use lazy_static::lazy_static;

use crate::{
    internal_config::Config,
    load_folder_structure::{get_flattened_files_structure, File, Folder},
    utils::{clone_extend_vec, remove_comments_from_code},
};

use self::{
    extract_file_content_exports::{
        extract_file_content_exports_from_clean_content, Export,
    },
    extract_file_content_imports::{
        extract_imports_from_clean_file_content, Import, ImportType,
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
    imports: IndexMap<String, Vec<Import>>,
    exports: Vec<Export>,
}

#[derive(Debug, Clone, Default)]
struct FileEdgesCache {
    edges_without_types: Vec<String>,
    edges: Vec<String>,
}

#[derive(Debug, Clone)]
struct ImportUsage {
    importer_path: String,
    imports: Vec<Import>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ResolveCacheKey {
    importer_path: Option<PathBuf>,
    import_path: PathBuf,
}

lazy_static! {
    static ref FILES_CACHE: Mutex<HashMap<String, File>> =
        Mutex::new(HashMap::new());
    static ref RESOLVE_CACHE: Mutex<HashMap<ResolveCacheKey, PathBuf>> =
        Mutex::new(HashMap::new());
    static ref IMPORTS_CACHE: Mutex<HashMap<String, IndexMap<String, Vec<Import>>>> =
        Mutex::new(HashMap::new());
    static ref EXPORTS_CACHE: Mutex<HashMap<String, Vec<Export>>> =
        Mutex::new(HashMap::new());
    static ref CLEAN_FILE_CONTENT_CACHE: Mutex<HashMap<String, String>> =
        Mutex::new(HashMap::new());
    pub static ref ALIASES: Mutex<HashMap<String, String>> =
        Mutex::new(HashMap::new());
    static ref ROOT_DIR: Mutex<String> = Mutex::new(String::from("."));
    static ref DEBUG_READ_EDGES_COUNT: Mutex<usize> = Mutex::new(0);
    static ref FILE_EDGES_CACHE: Mutex<HashMap<String, FileEdgesCache>> =
        Mutex::new(HashMap::new());
    pub static ref USED_FILES: Mutex<HashMap<String, FileDepsInfo>> =
        Mutex::new(HashMap::new());
    static ref REVERSE_IMPORTS: Mutex<HashMap<String, Vec<ImportUsage>>> =
        Mutex::new(HashMap::new());
    static ref FILE_DEPS_RESULT_CACHE: Mutex<HashMap<String, DepsResult>> =
        Mutex::new(HashMap::new());
}

pub fn _setup_test() {
    DEPS_CACHE.lock().unwrap().clear();
    FILES_CACHE.lock().unwrap().clear();
    RESOLVE_CACHE.lock().unwrap().clear();
    IMPORTS_CACHE.lock().unwrap().clear();
    EXPORTS_CACHE.lock().unwrap().clear();
    CLEAN_FILE_CONTENT_CACHE.lock().unwrap().clear();
    ALIASES.lock().unwrap().clear();
    *ROOT_DIR.lock().unwrap() = String::from(".");
    *DEBUG_READ_EDGES_COUNT.lock().unwrap() = 0;
    FILE_EDGES_CACHE.lock().unwrap().clear();
    USED_FILES.lock().unwrap().clear();
    REVERSE_IMPORTS.lock().unwrap().clear();
    FILE_DEPS_RESULT_CACHE.lock().unwrap().clear();
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn file_stem_to_string(path: &Path) -> Result<String, String> {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .ok_or_else(|| {
            format!(
                "TS: Path does not have a valid file stem: {}",
                path.display()
            )
        })
}

fn file_name_to_string(path: &Path) -> Result<String, String> {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| {
            format!(
                "TS: Path does not have a valid file name: {}",
                path.display()
            )
        })
}

fn load_file_from_cache(file_path: &PathBuf) -> Result<File, String> {
    let file_path_string = path_to_string(file_path);

    if let Some(file) = FILES_CACHE.lock().unwrap().get(&file_path_string).cloned() {
        return Ok(file);
    }

    let new_file = load_file_from_path(file_path)?;

    FILES_CACHE
        .lock()
        .unwrap()
        .insert(file_path_string, new_file.clone());

    Ok(new_file)
}

fn get_file_deps_result(file_path: &Path) -> Result<DepsResult, String> {
    let file_path_string = path_to_string(file_path);

    if let Some(file) = FILE_DEPS_RESULT_CACHE
        .lock()
        .unwrap()
        .get(&file_path_string)
        .cloned()
    {
        return Ok(file);
    }

    let deps = get_node_deps(
        &file_path_string,
        &mut |path| get_file_edges(path, true),
        None,
        false,
        false,
    )?;

    FILE_DEPS_RESULT_CACHE
        .lock()
        .unwrap()
        .insert(file_path_string, deps.clone());

    Ok(deps)
}

pub fn warm_file_deps_results_for_paths(
    file_paths: &[String],
) -> Result<(), String> {
    let mut sorted_file_paths = file_paths.to_vec();
    sorted_file_paths.sort();
    sorted_file_paths.dedup();

    for path in sorted_file_paths {
        get_file_deps_result(Path::new(&path))?;
    }

    Ok(())
}

fn get_resolved_path(path: &Path) -> Result<Option<PathBuf>, String> {
    get_resolved_path_from(None, path)
}

fn get_resolved_path_from(
    importer_path: Option<&Path>,
    path: &Path,
) -> Result<Option<PathBuf>, String> {
    let path_string = path_to_string(path);
    let is_relative_import = importer_path.is_some()
        && (path_string.starts_with("./") || path_string.starts_with("../"));

    if !is_relative_import
        && !ALIASES.lock().unwrap().iter().any(|(alias, replace)| {
            path.starts_with(alias) || path.starts_with(replace)
        })
    {
        return Ok(None);
    }

    if path_string.contains("?") {
        return Ok(None);
    }

    let cache_key = ResolveCacheKey {
        importer_path: if is_relative_import {
            importer_path.map(Path::to_path_buf)
        } else {
            None
        },
        import_path: path.to_path_buf(),
    };

    if let Some(resolved_path) = RESOLVE_CACHE.lock().unwrap().get(&cache_key) {
        return Ok(Some(resolved_path.clone()));
    }

    let unresolved_file_path = if is_relative_import {
        let importer_dir = importer_path
            .and_then(Path::parent)
            .unwrap_or_else(|| Path::new("."));
        normalize_relative_path(&importer_dir.join(path))
    } else {
        PathBuf::from(replace_aliases(&path_string))
    };

    let unresolved_file_path_string = path_to_string(&unresolved_file_path);

    let file_abs_path = format!(
        "{}{}",
        ROOT_DIR.lock().unwrap().clone(),
        unresolved_file_path_string.trim_start_matches('.')
    );

    let file_exists = FILES_CACHE
        .lock()
        .unwrap()
        .contains_key(&unresolved_file_path_string)
        || PathBuf::from(file_abs_path.clone()).is_file();

    if !file_exists {
        let file_name = file_name_to_string(&unresolved_file_path)?;
        let file_name_start_with_uppercase = file_name
            .chars()
            .next()
            .map(|char| char.is_uppercase())
            .unwrap_or(false);

        let test_extensions = if file_name_start_with_uppercase {
            vec![".tsx", ".ts", "/index.tsx", "/index.ts"]
        } else {
            vec![".ts", ".tsx", "/index.ts", "/index.tsx"]
        };

        for paths_to_try in &test_extensions {
            let cache_name =
                format!("{}{}", unresolved_file_path_string, paths_to_try);

            let file_is_in_cache =
                FILES_CACHE.lock().unwrap().contains_key(&cache_name);

            let load_path = if file_is_in_cache {
                PathBuf::from(&cache_name)
            } else {
                PathBuf::from(format!("{}{}", file_abs_path, paths_to_try))
            };

            if let Ok(loaded_file) = load_file_from_cache(&load_path) {
                let result_path = PathBuf::from(&cache_name);

                if !file_is_in_cache {
                    FILES_CACHE.lock().unwrap().insert(cache_name, loaded_file);
                }

                RESOLVE_CACHE
                    .lock()
                    .unwrap()
                    .insert(cache_key.clone(), result_path.clone());

                return Ok(Some(result_path));
            }
        }

        Err(format!(
            "TS: Can't resolve path: {:?}",
            unresolved_file_path
        ))
    } else {
        RESOLVE_CACHE
            .lock()
            .unwrap()
            .insert(cache_key, unresolved_file_path.clone());

        Ok(Some(unresolved_file_path))
    }
}

fn normalize_relative_path(path: &Path) -> PathBuf {
    let mut normalized_path = PathBuf::from(".");

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => normalized_path.push(part),
            Component::ParentDir => {
                if normalized_path == Path::new(".") || !normalized_path.pop() {
                    normalized_path.push("..");
                } else if normalized_path.as_os_str().is_empty() {
                    normalized_path.push(".");
                }
            }
            Component::Prefix(prefix) => {
                normalized_path = PathBuf::from(prefix.as_os_str());
            }
            Component::RootDir => {
                normalized_path = PathBuf::from(component.as_os_str());
            }
        }
    }

    normalized_path
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
) -> Result<IndexMap<String, Vec<Import>>, String> {
    if let Some(imports) = IMPORTS_CACHE.lock().unwrap().get(resolved_path).cloned()
    {
        return Ok(imports);
    }

    if let Some(file_content) = get_clean_file_content(resolved_path)? {
        let edges_imports = normalize_imports(
            extract_imports_from_clean_file_content(&file_content)?,
            resolved_path,
        )?;

        IMPORTS_CACHE
            .lock()
            .unwrap()
            .insert(resolved_path.to_string(), edges_imports.clone());

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

        let resolved_path_string = path_to_string(&resolved_path);
        let edges_imports = get_file_imports(&resolved_path_string)?;

        let mut non_type_edges = IndexSet::new();
        let mut edges = IndexSet::new();

        let files_cache = FILES_CACHE.lock().unwrap();

        for (import_name, imports) in &edges_imports {
            let resolved_path = if files_cache.contains_key(import_name) {
                Some(PathBuf::from(import_name))
            } else {
                get_resolved_path_from(
                    Some(Path::new(&resolved_path_string)),
                    Path::new(import_name),
                )?
            };

            if let Some(resolved_path) = resolved_path {
                let path_str = path_to_string(&resolved_path);

                for import in imports {
                    match import.values {
                        ImportType::Type(_) => {
                            edges.insert(path_str.clone());
                        }
                        _ => {
                            non_type_edges.insert(path_str.clone());
                            edges.insert(path_str.clone());
                        }
                    }
                }
            }
        }

        let edges = edges.into_iter().collect::<Vec<_>>();
        let non_type_edges = non_type_edges.into_iter().collect::<Vec<_>>();

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

fn get_clean_file_content(resolved_path: &str) -> Result<Option<String>, String> {
    if let Some(content) = CLEAN_FILE_CONTENT_CACHE
        .lock()
        .unwrap()
        .get(resolved_path)
        .cloned()
    {
        return Ok(Some(content));
    }

    let file_content = match get_file_content(resolved_path)? {
        Some(file_content) => file_content,
        None => return Ok(None),
    };

    let clean_content = remove_comments_from_code(&file_content);

    CLEAN_FILE_CONTENT_CACHE
        .lock()
        .unwrap()
        .insert(resolved_path.to_string(), clean_content.clone());

    Ok(Some(clean_content))
}

fn visit_file(
    resolved_path: &Path,
    result: &mut HashMap<String, FileDepsInfo>,
) -> Result<(), String> {
    let resolved_path_string = path_to_string(resolved_path);

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
    let exports = get_file_exports(resolved_path_string)?;
    let imports = get_file_imports(resolved_path_string)?;

    if exports.is_empty() && imports.is_empty() {
        return Ok(FileDepsInfo::default());
    }

    Ok(FileDepsInfo { exports, imports })
}

fn get_file_exports(resolved_path: &str) -> Result<Vec<Export>, String> {
    if let Some(exports) = EXPORTS_CACHE.lock().unwrap().get(resolved_path).cloned()
    {
        return Ok(exports);
    }

    if let Some(file_content) = get_clean_file_content(resolved_path)? {
        let exports =
            extract_file_content_exports_from_clean_content(&file_content)?;

        EXPORTS_CACHE
            .lock()
            .unwrap()
            .insert(resolved_path.to_string(), exports.clone());

        Ok(exports)
    } else {
        Ok(Vec::new())
    }
}

fn normalize_imports(
    imports: Vec<Import>,
    importer_path: &str,
) -> Result<IndexMap<String, Vec<Import>>, String> {
    let mut normalized_imports: IndexMap<String, Vec<Import>> = IndexMap::new();

    for new_import in imports {
        let resolved_import_name = if new_import.values == ImportType::Glob {
            None
        } else {
            get_resolved_path_from(
                Some(Path::new(importer_path)),
                &new_import.import_path,
            )?
        };

        let use_name = pb_to_string(
            resolved_import_name.unwrap_or(new_import.import_path.clone()),
        );

        match &new_import.values {
            ImportType::Glob => {
                let normalized_glob = if new_import.import_path.starts_with("/") {
                    format!(".{}", path_to_string(&new_import.import_path))
                } else {
                    path_to_string(&new_import.import_path)
                };

                let glob = globset::Glob::new(&normalized_glob)
                    .map_err(|err| {
                        format!(
                            "TS: Invalid import glob '{}': {}",
                            normalized_glob, err
                        )
                    })?
                    .compile_matcher();

                for file in FILES_CACHE.lock().unwrap().values() {
                    if glob.is_match(file.relative_path.as_str()) {
                        normalized_imports
                            .entry(file.relative_path.clone())
                            .or_default()
                            .push(Import {
                                import_path: PathBuf::from(
                                    file.relative_path.clone(),
                                ),
                                line: new_import.line,
                                values: ImportType::All,
                            });
                    }
                }
            }
            _ => {
                let entries = normalized_imports.entry(use_name).or_default();

                let merged = try_merge_import(entries, &new_import);

                if !merged {
                    entries.push(new_import);
                }
            }
        }
    }

    Ok(normalized_imports)
}

fn try_merge_import(entries: &mut [Import], new_import: &Import) -> bool {
    for existing in entries.iter_mut() {
        match (&existing.values, &new_import.values) {
            (ImportType::All | ImportType::Glob, _) => {
                return true;
            }
            (_, ImportType::All) => {
                existing.values = ImportType::All;
                existing.line = new_import.line;
                existing.import_path = new_import.import_path.clone();
                return true;
            }
            (ImportType::Named(current), ImportType::Named(new)) => {
                existing.values = ImportType::Named(clone_extend_vec(current, new));
                existing.line = new_import.line;
                existing.import_path = new_import.import_path.clone();
                return true;
            }
            (ImportType::Type(current), ImportType::Type(new)) => {
                existing.values = ImportType::Type(clone_extend_vec(current, new));
                existing.line = new_import.line;
                existing.import_path = new_import.import_path.clone();
                return true;
            }
            (
                ImportType::SideEffect | ImportType::Dynamic,
                ImportType::SideEffect | ImportType::Dynamic | ImportType::Named(_),
            ) => {
                existing.values = new_import.values.clone();
                existing.line = new_import.line;
                existing.import_path = new_import.import_path.clone();
                return true;
            }
            (ImportType::Named(_), ImportType::SideEffect | ImportType::Dynamic) => {
                return true;
            }
            _ => {}
        }
    }
    false
}

fn pb_to_string(import_path: PathBuf) -> String {
    path_to_string(&import_path)
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
        basename: file_stem_to_string(path)?,
        name_with_ext: file_name_to_string(path)?,
        content: file_content,
        extension: path
            .extension()
            .map(|ext| ext.to_string_lossy().into_owned()),
        relative_path: path_to_string(path),
    };

    Ok(file)
}

fn get_used_project_files_deps_info(
    entry_points: Vec<PathBuf>,
    flattened_root_structure: HashMap<String, File>,
    aliases: HashMap<String, String>,
) -> Result<(), String> {
    let mut result: HashMap<String, FileDepsInfo> = HashMap::new();

    let (mut non_glob_entry_points, glob_entry_points): (
        Vec<PathBuf>,
        Vec<PathBuf>,
    ) = entry_points
        .into_iter()
        .partition(|entry| !path_to_string(entry).contains('*'));

    for entry in glob_entry_points {
        let glob_pattern = path_to_string(&entry);
        let glob = globset::Glob::new(&glob_pattern)
            .map_err(|err| {
                format!(
                    "TS: Invalid unused_exports_entry_points glob '{}': {}",
                    glob_pattern, err
                )
            })?
            .compile_matcher();

        for file in flattened_root_structure.values() {
            if glob.is_match(file.relative_path.as_str()) {
                non_glob_entry_points
                    .push(PathBuf::from(file.relative_path.clone()));
            }
        }
    }

    FILES_CACHE.lock().unwrap().extend(flattened_root_structure);

    *ALIASES.lock().unwrap() = aliases;

    let mut seen_entry_points = HashSet::new();

    for entry in non_glob_entry_points
        .into_iter()
        .filter(|entry| seen_entry_points.insert(path_to_string(entry)))
    {
        if let Some(resolved_path) = get_resolved_path(&entry)? {
            visit_file(&resolved_path, &mut result)?;
        } else {
            return Err(format!(
                "TS: Can't resolve unused_exports_entry_points path: {:?}",
                entry
            ));
        }
    }

    *REVERSE_IMPORTS.lock().unwrap() = build_reverse_imports(&result);
    *USED_FILES.lock().unwrap() = result;

    Ok(())
}

fn build_reverse_imports(
    used_files: &HashMap<String, FileDepsInfo>,
) -> HashMap<String, Vec<ImportUsage>> {
    let mut reverse_imports: HashMap<String, Vec<ImportUsage>> = HashMap::new();

    for (importer_path, deps_info) in used_files {
        for (imported_path, imports) in &deps_info.imports {
            reverse_imports
                .entry(imported_path.clone())
                .or_default()
                .push(ImportUsage {
                    importer_path: importer_path.clone(),
                    imports: imports.clone(),
                });
        }
    }

    reverse_imports
}

pub fn load_used_project_files_deps_info_from_cfg(
    config: &Config,
    root_structure: &Folder,
    root_path: &Path,
) -> Result<(), String> {
    let unused_exports_entry_points = config
        .ts_config
        .as_ref()
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

    *ROOT_DIR.lock().unwrap() = path_to_string(root_path);

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
