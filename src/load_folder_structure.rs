use globset::Glob;
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
    pub childs: Vec<FolderChild>,
}

pub fn load_folder_structure(
    path: &Path,
    config: &Config,
    root: &PathBuf,
    is_root: bool,
) -> Result<Folder, String> {
    let mut childs: Vec<FolderChild> = vec![];

    let mut builder = globset::GlobSetBuilder::new();

    for pattern in &config.ignore {
        builder.add(Glob::new(pattern).unwrap());
    }

    let ignore_paths_set = builder.build().unwrap();

    for entry in path.read_dir().map_err(|err| {
        format!(
            "Error reading directory: {}, Error: {}",
            path.to_str().unwrap_or("invalid path"),
            err
        )
    })? {
        let entry = entry.unwrap();
        let path = entry.path();

        let relative_path = path.strip_prefix(root).unwrap();

        if !config.ignore.is_empty() && ignore_paths_set.is_match(relative_path) {
            continue;
        }

        if path.is_dir() {
            if is_root
                && config.root_folder.folder_rules.is_empty()
                && config
                    .root_folder
                    .sub_folders_config
                    .get(&format!(
                        "/{}",
                        path.file_name().unwrap().to_str().unwrap()
                    ))
                    .is_none()
            {
                continue;
            }

            childs.push(FolderChild::Folder(load_folder_structure(
                &path, config, root, false,
            )?));
        } else {
            let extension =
                path.extension().map(|s| s.to_str().unwrap().to_string());

            let file = File {
                basename: path.file_stem().unwrap().to_str().unwrap().to_string(),
                name_with_ext: path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
                content: get_file_content(config, &extension, path.clone()),
                extension,
                relative_path: format!("./{}", relative_path.to_str().unwrap()),
            };

            childs.push(FolderChild::FileChild(file));
        }
    }

    let folder_name = match path.file_name() {
        Some(name) => name.to_str().unwrap().to_string(),
        None => {
            let name = path.to_str().unwrap_or("invalid path");

            if name == "." {
                name.to_string()
            } else {
                return Err(format!("Error getting folder name: {}", name));
            }
        }
    };

    Ok(Folder {
        name: folder_name,
        childs,
    })
}

fn get_file_content(
    config: &Config,
    extension: &Option<String>,
    path: PathBuf,
) -> Option<String> {
    if let Some(extension) = extension {
        if config.analyze_content_of_files_types.contains(extension) {
            return Some(read_to_string(path).unwrap());
        }
    }

    None
}

pub fn get_flattened_files_structure(folder: &Folder) -> HashMap<String, File> {
    let mut result: HashMap<String, File> = HashMap::new();

    for child in &folder.childs {
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
                }],
                unexpected_files_error_msg: None,
                unexpected_folders_error_msg: None,
                one_of_blocks: OneOfBlocks::default(),
                optional: false,
                sub_folders_config: HashMap::new(),
            },
            ts_config: None,
        };

        let root = PathBuf::from("./src/fixtures/ignore_folder");

        let folder = load_folder_structure(&root, &config, &root, true);

        assert_debug_snapshot!(folder);
    }

    #[test]
    fn analyze_content_of_files_types() {
        let config = Config {
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
                }],
                unexpected_files_error_msg: None,
                unexpected_folders_error_msg: None,
                one_of_blocks: OneOfBlocks::default(),
                optional: false,
                sub_folders_config: HashMap::new(),
            },
            ts_config: None,
        };

        let root = PathBuf::from("./src/fixtures/analyze_file_contents");

        let folder = load_folder_structure(&root, &config, &root, true);

        assert_debug_snapshot!(folder);
    }

    #[test]
    fn ignore_unconfigured_folder() {
        let config = Config {
            analyze_content_of_files_types: vec!["js".to_string()],
            ignore: HashSet::from_iter(vec![".DS_Store".to_string()]),
            root_folder: FolderConfig {
                allow_unexpected_files: true,
                allow_unexpected_folders: true,
                file_rules: vec![],
                folder_rules: vec![],
                unexpected_files_error_msg: None,
                unexpected_folders_error_msg: None,
                one_of_blocks: OneOfBlocks::default(),
                optional: false,
                sub_folders_config: HashMap::new(),
            },
            ts_config: None,
        };

        let root = PathBuf::from("./src/fixtures/analyze_file_contents");

        let folder = load_folder_structure(&root, &config, &root, true);

        assert_debug_snapshot!(folder);
    }
}
