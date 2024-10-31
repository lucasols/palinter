mod analyze_ts_deps;
mod check_folders;
mod cli;
mod internal_config;
mod load_folder_structure;
mod parse_config_file;
mod test_config;
mod test_utils;
mod utils;

use std::{path::PathBuf, process};

use analyze_ts_deps::circular_deps::get_detailed_file_circular_deps_result;
use check_folders::{check_root_folder, Problems};

use cli::{get_cli_command, CliCommand};
use internal_config::{get_config, Config};
use load_folder_structure::load_folder_structure;
use parse_config_file::parse_config_file;
use test_config::test_config;

use crate::analyze_ts_deps::load_used_project_files_deps_info_from_cfg;

fn main() {
    match get_cli_command() {
        CliCommand::CircularDeps {
            file_name,
            cfg_path,
            root,
            truncate,
        } => {
            let parsed_config = match parse_config_file(&cfg_path) {
                Ok(config) => config,
                Err(err) => {
                    println!(
                        "❌ Error parsing config file: {}, {}",
                        cfg_path.to_str().unwrap(),
                        err
                    );
                    std::process::exit(1);
                }
            };

            let config = get_config(&parsed_config).unwrap();

            if let Err(err) = get_detailed_file_circular_deps_result(
                &file_name, &root, config, truncate,
            ) {
                eprintln!("❌ Error getting circular deps: {}", err);

                std::process::exit(1);
            }
        }

        CliCommand::TestConfig {
            test_cases_folder,
            cfg_path,
            fix_errors,
        } => match test_config(&test_cases_folder, &cfg_path, fix_errors) {
            Ok(success_msg) => println!("{}", success_msg),
            Err(err) => {
                eprintln!("❌ Error testing config: {}", err);
                std::process::exit(1);
            }
        },

        CliCommand::Lint {
            root,
            cfg_path,
            allow_warnings,
        } => {
            let parsed_config = match parse_config_file(&cfg_path) {
                Ok(config) => config,
                Err(err) => {
                    println!(
                        "❌ Error parsing config file '{}': {}",
                        cfg_path.to_str().unwrap(),
                        err
                    );
                    std::process::exit(1);
                }
            };

            let config = get_config(&parsed_config).unwrap();

            lint(config, root, allow_warnings);
        }
    }
}

fn lint(config: Config, root: PathBuf, allow_warnings: bool) {
    let measure_time = std::time::Instant::now();

    let root_structure = match load_folder_structure(&root, &config, &root, true) {
        Ok(root_structure) => root_structure,
        Err(err) => {
            eprintln!("❌ Error loading folder structure: {}", err);
            process::exit(1);
        }
    };

    if let Err(err) =
        load_used_project_files_deps_info_from_cfg(&config, &root_structure, &root)
    {
        eprintln!("❌ Error getting used files deps info: {}", err);
        std::process::exit(1);
    };

    if let Err(Problems { errors, warnings }) =
        check_root_folder(&config, &root_structure, false, allow_warnings)
    {
        let mut should_exit = false;

        if !errors.is_empty() {
            should_exit = true;
            eprintln!(
                "❌ Errors found in the project:\n\n{}\n\n",
                errors.join("\n\n")
            );
        }

        if !warnings.is_empty() {
            eprintln!(
                "🟠 Warnings found in the project:\n\n{}\n\n",
                warnings.join("\n\n")
            );
        }

        if should_exit {
            std::process::exit(1);
        }
    }

    println!("\n✨ The project architecture is valid!");
    println!("⌛ time: {:.3}s", measure_time.elapsed().as_secs_f32());
}
