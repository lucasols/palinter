mod parse_config_file;
mod check_files;
mod internal_config;
mod expect_checks;

use parse_config_file::parse_config;

fn main() {
    // TODO: read config file

    // TODO: parse config

    // TODO: run lint with config

    let config = parse_config("./tests/fixtures/config1.json");

    println!("{:#?}", config);
}
