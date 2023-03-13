use globset::Glob;
use std::path::PathBuf;

use crate::internal_config::Config;

#[derive(Debug)]
pub struct File {
    pub basename: String,
    pub name_with_ext: String,
    pub content: String,
    pub extension: Option<String>,
}

#[derive(Debug)]
pub enum FolderChild {
    FileChild(File),
    Folder(Folder),
}

#[derive(Debug)]
pub struct Folder {
    pub name: String,
    pub childs: Vec<FolderChild>,
}

pub fn load_folder_tree(path: &PathBuf, config: &Config, root: &PathBuf) -> Folder {
    let mut childs: Vec<FolderChild> = vec![];

    let mut builder = globset::GlobSetBuilder::new();

    for pattern in &config.ignore {
        builder.add(Glob::new(pattern).unwrap());
    }

    let ignore_paths_set = builder.build().unwrap();

    for entry in path.read_dir().unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        let relative_path = path.strip_prefix(root).unwrap();

        if !config.ignore.is_empty() && ignore_paths_set.is_match(relative_path) {
            continue;
        }

        if path.is_dir() {
            childs.push(FolderChild::Folder(load_folder_tree(&path, config, root)));
        } else {
            let extension = path.extension().map(|s| s.to_str().unwrap().to_string());

            let file = File {
                basename: path.file_stem().unwrap().to_str().unwrap().to_string(),
                name_with_ext: path.file_name().unwrap().to_str().unwrap().to_string(),
                content: if let Some(extensions) = &config.analyze_content_of_files_types {
                    if let Some(extension) = &extension {
                        if extensions.contains(extension) {
                            std::fs::read_to_string(&path).unwrap()
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                },
                extension,
            };

            childs.push(FolderChild::FileChild(file));
        }
    }

    Folder {
        name: path.file_name().unwrap().to_str().unwrap().to_string(),
        childs,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use insta::assert_debug_snapshot;

    use crate::internal_config::{FolderConfig, OneOfBlocks};

    use super::*;

    #[test]
    fn ignore_folders() {
        let config = Config {
            analyze_content_of_files_types: None,
            ignore: HashSet::from_iter(vec![
                "dist".to_string(),
                "node_modules".to_string(),
                ".DS_Store".to_string(),
            ]),
            root_folder: FolderConfig {
                allow_unconfigured_files: true,
                allow_unconfigured_folders: true,
                file_rules: vec![],
                folder_rules: vec![],
                one_of_blocks: OneOfBlocks::default(),
                optional: false,
                sub_folders_config: HashMap::new(),
            },
        };

        let root = PathBuf::from("./src/fixtures/ignore_folder");

        let folder = load_folder_tree(&root, &config, &root);

        assert_debug_snapshot!(folder);
    }

    #[test]
    fn analyze_content_of_files_types() {
        let config = Config {
            analyze_content_of_files_types: Some(vec!["js".to_string()]),
            ignore: HashSet::from_iter(vec![".DS_Store".to_string()]),
            root_folder: FolderConfig {
                allow_unconfigured_files: true,
                allow_unconfigured_folders: true,
                file_rules: vec![],
                folder_rules: vec![],
                one_of_blocks: OneOfBlocks::default(),
                optional: false,
                sub_folders_config: HashMap::new(),
            },
        };

        let root = PathBuf::from("./src/fixtures/analyze_file_contents");

        let folder = load_folder_tree(&root, &config, &root);

        assert_debug_snapshot!(folder);
    }
}
