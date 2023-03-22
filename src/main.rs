mod analyze_ts_deps;
mod check_folders;
mod internal_config;
mod load_folder_structure;
mod parse_config_file;
mod test_utils;
mod utils;

use std::path::PathBuf;

use analyze_ts_deps::circular_deps::get_detailed_file_circular_deps_result;
use check_folders::check_root_folder;
use clap::{arg, command, value_parser, Command};
use internal_config::{get_config, Config};
use load_folder_structure::load_folder_structure;
use parse_config_file::parse_config_file;

use crate::analyze_ts_deps::load_used_project_files_deps_info_from_cfg;

fn main() {
    let cli = command!()
        .arg(
            arg!(-c --config <config> "Path to the config file")
                .default_value("palinter.yaml")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(-r --root <root> "Path to the root folder of the project")
                .default_value(".")
                .value_parser(value_parser!(PathBuf)),
        )
        .subcommand(
            Command::new("circular-deps")
                .about("Check for circular dependencies in a file")
                .arg(
                    arg!([file] "Path to the file to check")
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    arg!(-c --config <config> "Path to the config file")
                        .default_value("palinter.yaml")
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    arg!(-r --root <root> "Path to the root folder of the project")
                        .default_value(".")
                        .value_parser(value_parser!(PathBuf)),
                ),
        )
        .get_matches();

    if let Some(matches) = cli.subcommand_matches("circular-deps") {
        if let Some(file_name) = matches.get_one::<PathBuf>("file") {
            let confg_path = matches.get_one::<PathBuf>("config").unwrap();

            let root = matches.get_one::<PathBuf>("root").unwrap();

            let parsed_config = match parse_config_file(confg_path) {
                Ok(config) => config,
                Err(err) => {
                    println!(
                        "❌ Error parsing config file: {}, {}",
                        confg_path.to_str().unwrap(),
                        err
                    );
                    std::process::exit(1);
                }
            };

            let config = get_config(&parsed_config).unwrap();

            if let Err(err) =
                get_detailed_file_circular_deps_result(file_name, root, config)
            {
                eprintln!("❌ Error getting circular deps: {}", err);

                std::process::exit(1);
            }
        }
    } else {
        let confg_path = cli.get_one::<PathBuf>("config").unwrap().clone();

        let root = cli.get_one::<PathBuf>("root").unwrap().clone();

        let parsed_config = match parse_config_file(&confg_path) {
            Ok(config) => config,
            Err(err) => {
                println!(
                    "❌ Error parsing config file '{}': {}",
                    confg_path.to_str().unwrap(),
                    err
                );
                std::process::exit(1);
            }
        };

        let config = get_config(&parsed_config).unwrap();

        lint(config, root);
    }
}

fn lint(config: Config, root: PathBuf) {
    let measure_time = std::time::Instant::now();

    let root_structure = match load_folder_structure(&root, &config, &root, true) {
        Ok(root_structure) => root_structure,
        Err(err) => {
            println!("❌ Error loading folder structure: {}", err);
            std::process::exit(1);
        }
    };

    if let Err(err) =
        load_used_project_files_deps_info_from_cfg(&config, &root_structure, &root)
    {
        println!("❌ Error getting used files deps info: {}", err);
        std::process::exit(1);
    };

    if let Err(err) = check_root_folder(&config, &root_structure) {
        println!(
            "❌ Errors found in the project:\n\n{}\n\n",
            err.join("\n\n")
        );
        std::process::exit(1);
    }

    println!("The project architecture is valid!");
    println!("Time: {:.3}s", measure_time.elapsed().as_secs_f32());
    std::process::exit(0);
}
