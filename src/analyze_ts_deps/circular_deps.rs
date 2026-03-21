use crate::{internal_config::Config, load_folder_structure};
use colored::Colorize;

use super::{
    extract_file_content_imports::{Import, ImportType},
    get_file_edges, get_file_imports, get_resolved_path,
    load_used_project_files_deps_info_from_cfg,
    modules_graph::get_node_deps,
    ALIASES, ROOT_DIR,
};

use std::path::Path;

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn get_imports_between(from_file: &str, to_file: &str) -> Vec<Import> {
    let inner = || -> Option<Vec<Import>> {
        let resolved = get_resolved_path(Path::new(from_file)).ok()??;
        let imports = get_file_imports(resolved.to_str()?).ok()?;
        Some(
            imports
                .get(to_file)?
                .iter()
                .filter(|i| !matches!(i.values, ImportType::Type(_)))
                .cloned()
                .collect(),
        )
    };

    inner().unwrap_or_default()
}

fn format_import_statement(import: &Import) -> String {
    let path = import.import_path.to_str().unwrap_or("?");

    let stmt = match &import.values {
        ImportType::Named(values) => {
            if values.len() > 3 {
                let truncated: Vec<&str> =
                    values.iter().take(3).map(|s| s.as_str()).collect();
                format!("import {{ {}, ... }} from '{}'", truncated.join(", "), path)
            } else {
                format!("import {{ {} }} from '{}'", values.join(", "), path)
            }
        }
        ImportType::All => format!("import * from '{}'", path),
        ImportType::Dynamic => format!("import('{}')", path),
        ImportType::SideEffect => format!("import '{}'", path),
        ImportType::Type(values) => {
            if values.len() > 3 {
                let truncated: Vec<&str> =
                    values.iter().take(3).map(|s| s.as_str()).collect();
                format!(
                    "import type {{ {}, ... }} from '{}'",
                    truncated.join(", "),
                    path
                )
            } else {
                format!("import type {{ {} }} from '{}'", values.join(", "), path)
            }
        }
        ImportType::Glob => {
            format!("import.meta.glob('{}')", path)
        }
    };

    format!("{}  :{}", stmt, import.line)
}

pub fn get_detailed_file_circular_deps_result(
    file_path: &Path,
    root_dir: &Path,
    config: Config,
    truncate: usize,
    only_direct_deps: bool,
) -> Result<(), String> {
    *ALIASES.lock().unwrap() = config
        .ts_config
        .as_ref()
        .map(|c| c.aliases.clone())
        .unwrap_or_default();
    *ROOT_DIR.lock().unwrap() = path_to_string(root_dir);

    let resolved_path = get_resolved_path(file_path)?
        .ok_or_else(|| format!("TS: Can't resolve path: {}", file_path.display()))?;
    let resolved_path_string = path_to_string(&resolved_path);

    let root_structure =
        load_folder_structure(root_dir, &config, &root_dir.to_path_buf(), true)?;

    load_used_project_files_deps_info_from_cfg(&config, &root_structure, root_dir)?;

    let result = get_node_deps(
        &resolved_path_string,
        &mut |path| get_file_edges(path, true),
        None,
        true,
        true,
    )?;

    if let Some(circular_deps) = result.circular_deps {
        let mut cdeps = circular_deps;

        let current_dep_path = format!("|{}|", resolved_path_string);

        cdeps.sort_by_key(|dep| {
            if dep.ends_with(&current_dep_path) {
                0
            } else {
                1
            }
        });

        if only_direct_deps {
            cdeps.retain(|dep| dep.contains(&current_dep_path));
        }

        let available_len = cdeps.len();
        cdeps.truncate(truncate);

        println!("🔁 {} circular deps found:", cdeps.len());

        if result.deps.contains(&resolved_path_string) {
            println!("\n{}", "Direct circular deps found".bright_yellow());
        }

        for dep in &cdeps {
            let parts = dep.split(" > ").collect::<Vec<&str>>();

            for (i, part) in parts.iter().enumerate() {
                let part_to_use = if part.starts_with('|') {
                    part.trim_matches('|')
                        .bright_yellow()
                        .to_string()
                        .replace("./", "#/")
                } else {
                    part.to_string()
                };

                if i == 0 {
                    println!("\n{} {}", ">>".dimmed(), part_to_use);
                } else {
                    println!("   {}{}", " ".repeat(i * 2), part_to_use);
                }

                if i < parts.len() - 1 {
                    let from_file = parts[i].trim_matches('|');
                    let to_file = parts[i + 1].trim_matches('|');
                    let imports = get_imports_between(from_file, to_file);
                    let indent = if i == 0 { 3 } else { 3 + i * 2 };

                    for import in &imports {
                        let formatted = format_import_statement(import);
                        println!(
                            "{}{}",
                            " ".repeat(indent),
                            format!("└── {}", formatted).dimmed()
                        );
                    }
                }
            }
        }

        if cdeps.len() < available_len {
            println!(
                "{}",
                format!("\n... and {} more", available_len - cdeps.len())
                    .bright_yellow()
            );
        }
        Ok(())
    } else {
        println!("No circular deps found");
        Ok(())
    }
}
