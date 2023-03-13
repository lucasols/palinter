mod check_folders;
mod internal_config;
mod load_folder_tree;
mod parse_config_file;
mod utils;

use std::path::PathBuf;

use check_folders::check_root_folder;
use clap::Parser;
use internal_config::get_config;
use load_folder_tree::load_folder_tree;
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

    let parsed_config = parse_config_file(&confg_path).unwrap();

    let config = get_config(&parsed_config).unwrap();

    let folder_tree = load_folder_tree(&root, &config, &root);

    match check_root_folder(&config, &folder_tree) {
        Ok(_) => {}
        Err(err) => {
            println!("❌ Errors found in the project:\n\n{}", err.join("\n\n"));
            std::process::exit(1);
        }
    }

    println!("✅ No errors detected!");
    std::process::exit(0);
}
