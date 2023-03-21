use std::{
    collections::HashMap,
    fs::read_to_string,
    path::{Path, PathBuf},
    sync::Mutex,
};

use indexmap::{IndexMap, IndexSet};
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
mod extract_file_content_exports;
mod extract_file_content_imports;
mod modules_graph;
pub mod ts_checks;

#[derive(Debug, Clone)]
pub struct FileDepsInfo {
    deps: IndexSet<PathBuf>,
    circular_deps: Option<Vec<String>>,
    imports: IndexMap<String, Import>,
    exports: Vec<Export>,
}

#[derive(Debug, Default)]
pub struct TsProjectCtx {
    pub files_cache: HashMap<String, File>,
    pub resolve_cache: HashMap<PathBuf, PathBuf>,
    pub imports_cache: HashMap<String, IndexMap<String, Import>>,
    pub aliases: HashMap<String, String>,
    pub root_dir: String,
    pub debug_read_edges_count: usize,
    pub file_edges_cache: HashMap<String, Vec<String>>,
}

lazy_static! {}

pub fn _setup_test() {
    DEPS_CACHE.lock().unwrap().clear();
}

fn load_file_from_cache(
    file_path: &PathBuf,
    cache: &mut TsProjectCtx,
) -> Result<File, String> {
    let from_cache = cache.files_cache.get(file_path.to_str().unwrap());

    let related_file = match from_cache {
        Some(file) => file.clone(),
        None => {
            let new_file = load_file_from_path(file_path)?;

            cache
                .files_cache
                .insert(file_path.to_str().unwrap().to_string(), new_file.clone());

            new_file
        }
    };

    Ok(related_file)
}

fn get_or_insert_file_dep_info(
    file_path: &Path,
    used_files_deps_info: &mut UsedFilesDepsInfo,
) -> Result<FileDepsInfo, String> {
    match used_files_deps_info
        .used_files
        .get(file_path.to_str().unwrap())
    {
        Some(deps_info) => Ok(deps_info.clone()),
        None => {
            let file_deps_info = get_file_deps_info(
                used_files_deps_info.ctx,
                &file_path.to_str().unwrap().to_string(),
            )?;

            used_files_deps_info.used_files.insert(
                file_path.to_str().unwrap().to_string(),
                file_deps_info.clone(),
            );

            Ok(file_deps_info)
        }
    }
}

fn get_resolved_path(
    path: &Path,
    ctx: &mut TsProjectCtx,
) -> Result<Option<PathBuf>, String> {
    if !ctx
        .aliases
        .iter()
        .any(|(alias, replace)| path.starts_with(alias) || path.starts_with(replace))
    {
        return Ok(None);
    }

    if let Some(resolved_path) = ctx.resolve_cache.get(path) {
        return Ok(Some(resolved_path.clone()));
    }

    let file_with_replaced_alias = replace_aliases(&ctx.aliases, path);

    let file_abs_path = format!(
        "{}{}",
        ctx.root_dir,
        file_with_replaced_alias
            .to_str()
            .unwrap()
            .trim_start_matches('.')
    );

    let file_exists = ctx
        .files_cache
        .contains_key(file_with_replaced_alias.to_str().unwrap())
        || PathBuf::from(file_abs_path.clone()).exists();

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
            vec!["tsx", "ts"]
        } else {
            vec!["ts", "tsx"]
        };

        for ext_to_try in &test_extensions {
            let cache_name = format!(
                "{}.{}",
                file_with_replaced_alias.to_str().unwrap(),
                ext_to_try
            );

            let file_is_in_cache = ctx.files_cache.contains_key(&cache_name);

            let new_path = if file_is_in_cache {
                PathBuf::from(cache_name)
            } else {
                PathBuf::from(format!("{}.{}", file_abs_path, ext_to_try))
            };

            let new_file = load_file_from_cache(&new_path, ctx);

            if new_file.is_ok() {
                ctx.resolve_cache
                    .insert(path.to_path_buf(), new_path.clone());

                return Ok(Some(new_path));
            }
        }

        return Err(format!(
            "TS: Can't find file with extensions .ts or .tsx for path: {:?}",
            file_with_replaced_alias
        ));
    } else {
        ctx.resolve_cache
            .insert(path.to_path_buf(), PathBuf::from(file_abs_path));

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

fn get_file_imports(
    resolved_path: &str,
    ctx: &mut TsProjectCtx,
) -> Result<IndexMap<String, Import>, String> {
    let from_cache = ctx.imports_cache.get(resolved_path);

    if let Some(imports) = from_cache {
        return Ok(imports.clone());
    }

    if let Some(file_content) = get_file_content(resolved_path, ctx)? {
        let edges_imports = normalize_imports(
            extract_imports_from_file_content(&file_content)?,
            ctx,
        )?;

        ctx.imports_cache
            .insert(resolved_path.to_string(), edges_imports.clone());

        Ok(edges_imports)
    } else {
        Ok(IndexMap::new())
    }
}

fn get_file_edges(
    unresolved_path: &str,
    ctx: &mut TsProjectCtx,
) -> Result<Vec<String>, String> {
    let resolved_path = get_resolved_path(Path::new(unresolved_path), ctx)?;

    if ctx.debug_read_edges_count > 2_000_000 {
        panic!("Too many edges read, probably infinite loop");
    }

    ctx.debug_read_edges_count += 1;

    if let Some(cached_edges) = ctx.file_edges_cache.get(unresolved_path) {
        return Ok(cached_edges.clone());
    }

    if let Some(resolved_path) = resolved_path {
        if let Some(resolved_path_ext) =
            resolved_path.extension().and_then(|s| s.to_str())
        {
            if resolved_path_ext != "ts" && resolved_path_ext != "tsx" {
                return Ok(vec![]);
            }
        }

        let edges_imports = get_file_imports(resolved_path.to_str().unwrap(), ctx)?;

        let edges: Vec<String> = edges_imports
            .values()
            .map(|import: &Import| -> Result<Option<PathBuf>, String> {
                if let Some(resolved_path) =
                    get_resolved_path(&import.import_path, ctx)?
                {
                    Ok(Some(resolved_path))
                } else {
                    Ok(None)
                }
            })
            .collect::<Result<Vec<Option<PathBuf>>, String>>()?
            .into_iter()
            .filter_map(|path| path.map(|p| p.to_str().unwrap().to_string()))
            .collect();

        ctx.file_edges_cache
            .insert(unresolved_path.to_string(), edges.clone());

        Ok(edges)
    } else {
        ctx.file_edges_cache
            .insert(unresolved_path.to_string(), vec![]);

        Ok(vec![])
    }
}

fn get_file_content(
    resolved_path: &str,
    cache: &mut TsProjectCtx,
) -> Result<Option<String>, String> {
    let related_file = load_file_from_cache(&PathBuf::from(resolved_path), cache)?;

    Ok(related_file.content)
}

fn visit_file(
    resolved_path: &Path,
    result: &mut HashMap<String, FileDepsInfo>,
    ctx: &mut TsProjectCtx,
) -> Result<(), String> {
    let resolved_path_string = resolved_path.to_str().unwrap().to_string();

    let file_deps_info = get_file_deps_info(ctx, &resolved_path_string)?;

    result.insert(resolved_path_string.clone(), file_deps_info);

    let edges = get_file_edges(&resolved_path_string, ctx)?;

    for edge in edges {
        let edge_path = PathBuf::from(edge.clone());

        if !result.contains_key(&edge) {
            visit_file(&edge_path, result, ctx)?;
        }
    }

    Ok(())
}

fn get_file_deps_info(
    ctx: &mut TsProjectCtx,
    resolved_path_string: &String,
) -> Result<FileDepsInfo, String> {
    let DepsResult {
        deps,
        circular_deps,
    } = get_node_deps(resolved_path_string, &mut |edge_id| {
        get_file_edges(edge_id, ctx)
    })?;

    let file_content = get_file_content(resolved_path_string, ctx)?;

    if let Some(file_content) = file_content {
        let exports = extract_file_content_exports(&file_content)?;

        let imports = get_file_imports(resolved_path_string, ctx)?;

        let file_deps_info = FileDepsInfo {
            deps: deps.iter().map(PathBuf::from).collect(),
            exports,
            circular_deps,
            imports,
        };

        Ok(file_deps_info)
    } else {
        Ok(FileDepsInfo {
            deps: IndexSet::new(),
            exports: vec![],
            circular_deps: None,
            imports: IndexMap::new(),
        })
    }
}

fn normalize_imports(
    imports: Vec<Import>,
    ctx: &mut TsProjectCtx,
) -> Result<IndexMap<String, Import>, String> {
    let mut normalized_imports: IndexMap<String, Import> = IndexMap::new();

    for import in imports {
        let resolved_import_name = get_resolved_path(&import.import_path, ctx)?;

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
    ctx: &mut TsProjectCtx,
) -> Result<HashMap<String, FileDepsInfo>, String> {
    let mut result: HashMap<String, FileDepsInfo> = HashMap::new();

    ctx.files_cache.extend(flattened_root_structure);

    ctx.aliases = aliases;

    for entry in entry_points {
        if let Some(resolved_path) = get_resolved_path(&entry, ctx)? {
            visit_file(&resolved_path, &mut result, ctx)?;
        }
    }

    Ok(result)
}

pub struct UsedFilesDepsInfo<'a> {
    used_files: HashMap<String, FileDepsInfo>,
    ctx: &'a mut TsProjectCtx,
}

pub fn get_used_project_files_deps_info_from_cfg<'a>(
    config: &'a Config,
    root_structure: &'a Folder,
    root_path: &'a Path,
    ts_ctx: &'a mut TsProjectCtx,
) -> Result<UsedFilesDepsInfo<'a>, String> {
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
        return Ok(UsedFilesDepsInfo {
            used_files: HashMap::default(),
            ctx: ts_ctx,
        });
    }

    ts_ctx.root_dir = root_path.to_str().unwrap().to_string();

    let flattened_root_structure = if !unused_exports_entry_points.is_empty() {
        get_flattened_files_structure(root_structure)
    } else {
        HashMap::default()
    };

    let used_files = get_used_project_files_deps_info(
        unused_exports_entry_points,
        flattened_root_structure,
        config
            .ts_config
            .as_ref()
            .map(|c| c.aliases.clone())
            .unwrap_or_default(),
        ts_ctx,
    )?;

    Ok(UsedFilesDepsInfo {
        used_files,
        ctx: ts_ctx,
    })
}

#[cfg(test)]
mod tests;
