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

use cli::{get_cli_args, CliCommand};
use internal_config::{get_config, Config};
use load_folder_structure::{count_files, load_folder_structure};
use parse_config_file::parse_config_file;
use test_config::test_config;

use crate::analyze_ts_deps::load_used_project_files_deps_info_from_cfg;

fn main() {
    let cli_args = get_cli_args();

    if let Some(threads) = &cli_args.threads {
        let threads = match threads.resolve() {
            Ok(threads) => threads,
            Err(err) => {
                eprintln!("❌ {}", err);
                std::process::exit(1);
            }
        };

        if let Err(err) = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
        {
            eprintln!("❌ Error configuring thread pool: {}", err);
            std::process::exit(1);
        }
    }

    match cli_args.command {
        CliCommand::CircularDeps {
            file_name,
            cfg_path,
            root,
            truncate,
            only_direct_deps,
        } => {
            let parsed_config = match parse_config_file(&cfg_path) {
                Ok(config) => config,
                Err(err) => {
                    println!(
                        "❌ Error parsing config file: {}, {}",
                        cfg_path.display(),
                        err
                    );
                    std::process::exit(1);
                }
            };

            let config = match get_config(&parsed_config) {
                Ok(config) => config,
                Err(err) => {
                    eprintln!("❌ Error building config: {}", err);
                    std::process::exit(1);
                }
            };

            if let Err(err) = get_detailed_file_circular_deps_result(
                &file_name,
                &root,
                config,
                truncate,
                only_direct_deps,
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
                        cfg_path.display(),
                        err
                    );
                    std::process::exit(1);
                }
            };

            let config = match get_config(&parsed_config) {
                Ok(config) => config,
                Err(err) => {
                    eprintln!("❌ Error building config: {}", err);
                    std::process::exit(1);
                }
            };

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

    let total_files_processed = count_files(&root_structure);

    println!("\n✨ The project architecture is valid!");
    println!("📄 files processed: {}", total_files_processed);
    println!("⌛ time: {:.3}s", measure_time.elapsed().as_secs_f32());
}
