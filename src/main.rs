mod config;

use config::parse_config;

fn main() {
    // TODO: read config file

    // TODO: parse config

    // TODO: run lint with config

    let config = parse_config("./tests/fixtures/config1.json");

    println!("{:#?}", config);
}
