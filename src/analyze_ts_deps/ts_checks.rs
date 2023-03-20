use std::collections::HashMap;

use crate::load_folder_structure::File;

use super::{extract_file_content_imports::ImportType, FileDepsInfo};

pub fn check_ts_not_have_unused_exports(
    file: &File,
    used_files_deps_info: &HashMap<String, FileDepsInfo>,
) -> Result<(), String> {
    let deps_info = used_files_deps_info.get(&file.path);

    if let Some(deps_info) = deps_info {
        let mut unused_exports = deps_info.exports.clone();

        for (other_used_file, other_deps_info) in used_files_deps_info {
            if unused_exports.is_empty() {
                break;
            }

            if other_used_file == &file.path {
                continue;
            }

            if let Some(related_import) = other_deps_info.imports.get(&file.path) {
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
