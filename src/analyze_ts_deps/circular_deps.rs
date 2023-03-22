use crate::{internal_config::Config, load_folder_structure};
use colored::Colorize;

use super::{
    get_file_edges, get_resolved_path, load_used_project_files_deps_info_from_cfg,
    modules_graph::get_node_deps, ALIASES, ROOT_DIR,
};

use std::path::Path;

pub fn get_detailed_file_circular_deps_result(
    file_path: &Path,
    root_dir: &Path,
    config: Config,
) -> Result<(), String> {
    *ALIASES.lock().unwrap() = config
        .ts_config
        .as_ref()
        .map(|c| c.aliases.clone())
        .unwrap_or_default();
    *ROOT_DIR.lock().unwrap() = root_dir.to_str().unwrap().to_string();

    let resolved_path = get_resolved_path(file_path)?.unwrap();

    let root_structure = match load_folder_structure(
        root_dir,
        &config,
        &root_dir.to_path_buf(),
        true,
    ) {
        Ok(root_structure) => root_structure,
        Err(err) => {
            println!("❌ Error loading folder structure: {}", err);
            std::process::exit(1);
        }
    };

    if let Err(err) = load_used_project_files_deps_info_from_cfg(
        &config,
        &root_structure,
        root_dir,
    ) {
        println!("❌ Error getting used files deps info: {}", err);
        std::process::exit(1);
    };

    let result = get_node_deps(
        &resolved_path.to_str().unwrap().to_string(),
        &mut get_file_edges,
        None,
        true,
        true,
    )?;

    if let Some(circular_deps) = result.circular_deps {
        let mut cdeps = circular_deps;

        let original_len = cdeps.len();

        cdeps.truncate(8);

        println!("🔁 Circular deps found:");

        for dep in &cdeps {
            let parts = dep.split(" > ").collect::<Vec<&str>>();

            for (i, part) in parts.iter().enumerate() {
                let part_to_use = if part.starts_with('|') {
                    part.bright_yellow().to_string()
                } else {
                    part.to_string()
                };

                if i == 0 {
                    println!("\n{} {}", ">>".dimmed(), part_to_use);
                } else {
                    println!("   {}{}", " ".repeat(i * 2), part_to_use);
                }
            }
        }

        if cdeps.len() < original_len {
            println!(
                "{}",
                format!("\n... and {} more", original_len - cdeps.len())
                    .bright_yellow()
            );
        }
        Ok(())
    } else {
        println!("No circular deps found");
        Ok(())
    }
}
