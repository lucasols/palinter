mod analyze_ts_deps;
mod check_folders;
mod internal_config;
mod load_folder_structure;
mod parse_config_file;
mod test_config;
mod test_utils;
mod utils;

use std::{path::PathBuf, process};

use analyze_ts_deps::circular_deps::get_detailed_file_circular_deps_result;
use check_folders::check_root_folder;
use clap::{arg, command, value_parser, Arg, Command};
use internal_config::{get_config, Config};
use load_folder_structure::load_folder_structure;
use parse_config_file::parse_config_file;
use test_config::test_config;

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
                )
                .arg(
                    arg!(-t --truncate <truncate> "Truncate the output to the first n elements")
                        .default_value("10")
                        .value_parser(value_parser!(usize)),
                ),
        )
        .subcommand(
            Command::new("test-config")
                .about("Test the config file with test cases")
                .arg(
                    arg!([test_cases_folder] "Path to the folder with the test cases")
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    arg!(-c --config <config> "Path to the config file")
                        .default_value("palinter.yaml")
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    Arg::new("fix-errors")
                        .help("Fix the errors in the test cases")
                        .long("fix-errors")
                        .short('f')
                        .num_args(0)
                        .required(false),
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

            if let Err(err) = get_detailed_file_circular_deps_result(
                file_name,
                root,
                config,
                *matches.get_one::<usize>("truncate").unwrap(),
            ) {
                eprintln!("❌ Error getting circular deps: {}", err);

                std::process::exit(1);
            }
        }
    } else if let Some(matches) = cli.subcommand_matches("test-config") {
        if let Some(test_case_dir) = matches.get_one::<PathBuf>("test_cases_folder")
        {
            let confg_path = matches.get_one::<PathBuf>("config").unwrap();

            let fix_errors = matches.contains_id("fix-errors");

            match test_config(test_case_dir, confg_path, fix_errors) {
                Ok(success_msg) => println!("{}", success_msg),
                Err(err) => {
                    eprintln!("❌ Error testing config: {}", err);
                    std::process::exit(1);
                }
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

    if let Err(err) = check_root_folder(&config, &root_structure) {
        eprintln!(
            "❌ Errors found in the project:\n\n{}\n\n",
            err.join("\n\n")
        );
        std::process::exit(1);
    }

    println!("\n✨ The project architecture is valid!");
    println!("⌛ time: {:.3}s", measure_time.elapsed().as_secs_f32());
}
