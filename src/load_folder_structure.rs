use globset::{Glob, GlobSet};
use rayon::prelude::*;
use std::{
    collections::HashMap,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use crate::internal_config::Config;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct File {
    pub basename: String,
    pub name_with_ext: String,
    pub content: Option<String>,
    pub extension: Option<String>,
    pub relative_path: String,
}

#[derive(Debug, PartialEq)]
pub enum FolderChild {
    FileChild(File),
    Folder(Folder),
}

#[derive(Debug, PartialEq)]
pub struct Folder {
    pub name: String,
    pub children: Vec<FolderChild>,
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn file_name_to_string(path: &Path) -> Result<String, String> {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| {
            format!("Error getting file name from path '{}'", path.display())
        })
}

fn file_stem_to_string(path: &Path) -> Result<String, String> {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .ok_or_else(|| {
            format!("Error getting file stem from path '{}'", path.display())
        })
}

pub fn load_folder_structure(
    path: &Path,
    config: &Config,
    root: &PathBuf,
    is_root: bool,
) -> Result<Folder, String> {
    let ignore_paths_set = build_ignore_paths_set(config, is_root)?;

    load_folder_structure_with_ignores(
        path,
        config,
        root,
        is_root,
        &ignore_paths_set,
    )
}

fn build_ignore_paths_set(
    config: &Config,
    is_root: bool,
) -> Result<GlobSet, String> {
    let mut builder = globset::GlobSetBuilder::new();

    for pattern in &config.ignore {
        builder.add(Glob::new(pattern).map_err(|err| {
            format!("Invalid ignore pattern '{}': {}", pattern, err)
        })?);
    }

    if is_root {
        builder.add(Glob::new("**/node_modules").map_err(|err| {
            format!("Invalid built-in ignore pattern '**/node_modules': {}", err)
        })?);
        builder.add(Glob::new("**/.git").map_err(|err| {
            format!("Invalid built-in ignore pattern '**/.git': {}", err)
        })?);
    }

    builder
        .build()
        .map_err(|err| format!("Error building ignore patterns: {}", err))
}

fn load_folder_structure_with_ignores(
    path: &Path,
    config: &Config,
    root: &PathBuf,
    is_root: bool,
    ignore_paths_set: &GlobSet,
) -> Result<Folder, String> {
    let mut child_paths = path
        .read_dir()
        .map_err(|err| {
            format!(
                "Error reading directory: {}, Error: {}",
                path.display(),
                err
            )
        })?
        .map(|entry| {
            entry.map(|entry| entry.path()).map_err(|err| {
                format!(
                    "Error reading directory entry in '{}': {}",
                    path.display(),
                    err
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    child_paths.sort_by_cached_key(|child_path| path_to_string(child_path));

    let children = child_paths
        .into_par_iter()
        .map(|child_path| {
            load_folder_child(child_path, config, root, is_root, ignore_paths_set)
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();

    let folder_name = match path.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => {
            let name = path_to_string(path);

            if name == "." {
                name
            } else {
                return Err(format!("Error getting folder name: {}", name));
            }
        }
    };

    Ok(Folder {
        name: folder_name,
        children,
    })
}

fn load_folder_child(
    path: PathBuf,
    config: &Config,
    root: &PathBuf,
    is_root: bool,
    ignore_paths_set: &GlobSet,
) -> Result<Option<FolderChild>, String> {
    let relative_path = path.strip_prefix(root).map_err(|err| {
        format!(
            "Error getting relative path for '{}' from '{}': {}",
            path.display(),
            root.display(),
            err
        )
    })?;

    if ignore_paths_set.is_match(relative_path) {
        return Ok(None);
    }

    if path.is_dir() {
        if should_skip_root_dir(&path, config, is_root)? {
            return Ok(None);
        }

        return load_folder_structure_with_ignores(
            &path,
            config,
            root,
            false,
            ignore_paths_set,
        )
        .map(FolderChild::Folder)
        .map(Some);
    }

    let extension = path.extension().map(|s| s.to_string_lossy().into_owned());

    Ok(Some(FolderChild::FileChild(File {
        basename: file_stem_to_string(&path)?,
        name_with_ext: file_name_to_string(&path)?,
        content: get_file_content(config, &extension, path.clone())?,
        extension,
        relative_path: format!("./{}", path_to_string(relative_path)),
    })))
}

fn should_skip_root_dir(
    path: &Path,
    config: &Config,
    is_root: bool,
) -> Result<bool, String> {
    Ok(is_root
        && config.root_folder.folder_rules.is_empty()
        && !config
            .root_folder
            .sub_folders_config
            .contains_key(&format!("/{}", file_name_to_string(path)?)))
}

fn get_file_content(
    config: &Config,
    extension: &Option<String>,
    path: PathBuf,
) -> Result<Option<String>, String> {
    if let Some(extension) = extension {
        if config.analyze_content_of_files_types.contains(extension) {
            return read_to_string(&path).map(Some).map_err(|err| {
                format!("Error reading file '{}': {}", path.display(), err)
            });
        }
    }

    Ok(None)
}

pub fn get_flattened_files_structure(folder: &Folder) -> HashMap<String, File> {
    let mut result: HashMap<String, File> = HashMap::new();

    for child in &folder.children {
        match child {
            FolderChild::FileChild(file) => {
                result.insert(file.clone().relative_path, file.clone());
            }
            FolderChild::Folder(folder) => {
                result.extend(get_flattened_files_structure(folder));
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use insta::assert_debug_snapshot;

    use crate::internal_config::{FolderConfig, FolderRule, OneOfBlocks};

    use super::*;

    #[test]
    fn ignore_folders() {
        let config = Config {
            allow_warnings: false,
            analyze_content_of_files_types: vec![],
            ignore: HashSet::from_iter(vec![
                "dist".to_string(),
                "node_modules".to_string(),
                ".DS_Store".to_string(),
            ]),
            root_folder: FolderConfig {
                allow_unexpected_files: true,
                allow_unexpected_folders: true,
                file_rules: vec![],
                folder_rules: vec![FolderRule {
                    conditions: crate::internal_config::AnyOr::Any,
                    expect: crate::internal_config::AnyNoneOr::Any,
                    error_msg: None,
                    non_recursive: false,
                    not_touch: false,
                    allow_unexpected_files: false,
                    allow_unexpected_folders: false,
                    is_warning: false,
                }],
                unexpected_files_error_msg: None,
                unexpected_folders_error_msg: None,
                unexpected_error_msg: None,
                append_error_msg: None,
                one_of_blocks: OneOfBlocks::default(),
                optional: false,
                sub_folders_config: HashMap::new(),
            },
            ts_config: None,
            error_msg_vars: None,
        };

        let root = PathBuf::from("./src/fixtures/ignore_folder");

        let folder = load_folder_structure(&root, &config, &root, true);

        assert_debug_snapshot!(folder);
    }

    #[test]
    fn ignore_builtin_folders_without_custom_ignore() {
        let config = Config {
            allow_warnings: false,
            analyze_content_of_files_types: vec![],
            ignore: HashSet::new(),
            root_folder: FolderConfig {
                allow_unexpected_files: true,
                allow_unexpected_folders: true,
                file_rules: vec![],
                folder_rules: vec![FolderRule {
                    conditions: crate::internal_config::AnyOr::Any,
                    expect: crate::internal_config::AnyNoneOr::Any,
                    error_msg: None,
                    non_recursive: false,
                    not_touch: false,
                    allow_unexpected_files: false,
                    allow_unexpected_folders: false,
                    is_warning: false,
                }],
                unexpected_files_error_msg: None,
                unexpected_folders_error_msg: None,
                unexpected_error_msg: None,
                append_error_msg: None,
                one_of_blocks: OneOfBlocks::default(),
                optional: false,
                sub_folders_config: HashMap::new(),
            },
            ts_config: None,
            error_msg_vars: None,
        };

        let root = PathBuf::from("./src/fixtures/ignore_folder");
        let folder = load_folder_structure(&root, &config, &root, true).unwrap();

        assert!(folder.children.iter().all(|child| {
            !matches!(
                child,
                FolderChild::Folder(folder) if folder.name == "node_modules"
                    || folder.name == ".git"
            )
        }));
    }

    #[test]
    fn analyze_content_of_files_types() {
        let config = Config {
            allow_warnings: false,
            analyze_content_of_files_types: vec!["js".to_string()],
            ignore: HashSet::from_iter(vec![".DS_Store".to_string()]),
            root_folder: FolderConfig {
                allow_unexpected_files: true,
                allow_unexpected_folders: true,
                file_rules: vec![],
                folder_rules: vec![FolderRule {
                    conditions: crate::internal_config::AnyOr::Any,
                    expect: crate::internal_config::AnyNoneOr::Any,
                    error_msg: None,
                    non_recursive: false,
                    not_touch: false,
                    allow_unexpected_files: false,
                    allow_unexpected_folders: false,
                    is_warning: false,
                }],
                unexpected_files_error_msg: None,
                unexpected_folders_error_msg: None,
                append_error_msg: None,
                unexpected_error_msg: None,
                one_of_blocks: OneOfBlocks::default(),
                optional: false,
                sub_folders_config: HashMap::new(),
            },
            ts_config: None,
            error_msg_vars: None,
        };

        let root = PathBuf::from("./src/fixtures/analyze_file_contents");

        let folder = load_folder_structure(&root, &config, &root, true);

        assert_debug_snapshot!(folder);
    }

    #[test]
    fn ignore_unconfigured_folder() {
        let config = Config {
            allow_warnings: false,
            analyze_content_of_files_types: vec!["js".to_string()],
            ignore: HashSet::from_iter(vec![".DS_Store".to_string()]),
            root_folder: FolderConfig {
                allow_unexpected_files: true,
                allow_unexpected_folders: true,
                file_rules: vec![],
                folder_rules: vec![],
                append_error_msg: None,
                unexpected_error_msg: None,
                unexpected_files_error_msg: None,
                unexpected_folders_error_msg: None,
                one_of_blocks: OneOfBlocks::default(),
                optional: false,
                sub_folders_config: HashMap::new(),
            },
            ts_config: None,
            error_msg_vars: None,
        };

        let root = PathBuf::from("./src/fixtures/analyze_file_contents");

        let folder = load_folder_structure(&root, &config, &root, true);

        assert_debug_snapshot!(folder);
    }
}
