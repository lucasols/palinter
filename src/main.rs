mod check_folders;
mod internal_config;
mod load_folder_structure;
mod parse_config_file;
mod utils;
mod analyse_ts_deps;

use std::path::PathBuf;

use check_folders::check_root_folder;
use clap::Parser;
use internal_config::get_config;
use load_folder_structure::load_folder_structure;
use parse_config_file::parse_config_file;

#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value = "palinter.yaml")]
    config: PathBuf,
    #[arg(short, long, default_value = ".")]
    root: PathBuf,
}

fn main() {
    let args = Cli::parse();

    let confg_path = args.config;

    let root = args.root;

    lint(confg_path, root);
}

fn lint(confg_path: PathBuf, root: PathBuf) {
    let measure_time = std::time::Instant::now();

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

    let root_structure = match load_folder_structure(&root, &config, &root, true) {
        Ok(root_structure) => root_structure,
        Err(err) => {
            println!("❌ Error loading folder structure: {}", err);
            std::process::exit(1);
        }
    };

    match check_root_folder(&config, &root_structure) {
        Ok(_) => {}
        Err(err) => {
            println!("❌ Errors found in the project:\n\n{}\n\n", err.join("\n\n"));
            std::process::exit(1);
        }
    }

    println!("The project architecture is valid!");
    println!("Time: {:.3}s", measure_time.elapsed().as_secs_f32());
    std::process::exit(0);
}
