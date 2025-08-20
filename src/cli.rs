use std::path::PathBuf;

use clap::{builder, Arg, ArgAction, ArgMatches, Command};

#[derive(Debug, PartialEq)]
pub enum CliCommand {
    CircularDeps {
        file_name: PathBuf,
        cfg_path: PathBuf,
        root: PathBuf,
        truncate: usize,
        only_direct_deps: bool,
    },
    TestConfig {
        test_cases_folder: PathBuf,
        cfg_path: PathBuf,
        fix_errors: bool,
    },
    Lint {
        root: PathBuf,
        cfg_path: PathBuf,
        allow_warnings: bool,
    },
}

fn get_clap_command() -> Command {
    Command::new("palinter")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Path to the config file")
                .default_value("palinter.yaml")
                .value_parser(builder::PathBufValueParser::new()),
        )
        .arg(
            Arg::new("root")
                .short('r')
                .long("root")
                 .help("Path to the root folder of the project")
                .default_value(".")
                .value_parser(builder::PathBufValueParser::new()),
        )
        .arg(
            Arg::new("allow-warnings")
                .long("allow-warnings")
                .action(ArgAction::SetTrue)
                .help("Show rules with `is_warning` set to true as warnings instead of errors")
        )
         .subcommand(
            Command::new("circular-deps")
                .about("Check for circular dependencies in a file")
                .arg(
                    Arg::new("file")
                         .required(true)
                        .value_parser(builder::PathBufValueParser::new())
                        .help("Path to the file to check"),
                )
                .arg(
                    Arg::new("config")
                        .short('c')
                        .long("config")
                         .default_value("palinter.yaml")
                        .value_parser(builder::PathBufValueParser::new())
                        .help("Path to the config file"),
                )
                .arg(
                    Arg::new("root")
                        .short('r')
                        .long("root")
                        .default_value(".")
                        .value_parser(builder::PathBufValueParser::new())
                        .help("Path to the root folder of the project"),
                )
                .arg(
                    Arg::new("truncate")
                        .short('t')
                        .long("truncate")
                        .default_value("10")
                        .value_parser(
                            builder::RangedU64ValueParser::<usize>::new().range(0..),
                        )
                        .help("Truncate the output to the first n elements"),
                )
                .arg(
                    Arg::new("only-direct-deps")
                        .short('D')
                        .long("only-direct-deps")
                        .action(ArgAction::SetTrue)
                        .help("Show only circular deps that include the target file"),
                ),
        )
        .subcommand(
            Command::new("test-config")
                .about("Test the config file with test cases")
                .arg(
                    Arg::new("test_cases_folder")
                        .required(true)
                        .value_parser(builder::PathBufValueParser::new())
                        .help("Path to the folder with the test cases"),
                )
                .arg(
                    Arg::new("config")
                        .short('c')
                        .long("config")
                        .default_value("palinter.yaml")
                        .value_parser(builder::PathBufValueParser::new())
                        .help("Path to the config file"),
                )
                .arg(
                    Arg::new("fix-errors")
                        .short('f')
                         .long("fix-errors")
                         .action(ArgAction::SetTrue)
                        .help("Fix the errors in the test cases"),
                ),
        )
}

fn get_cli_cmd_from_matches(matches: &ArgMatches) -> CliCommand {
    match matches.subcommand() {
        Some(("circular-deps", sub_matches)) => CliCommand::CircularDeps {
            file_name: sub_matches.get_one::<PathBuf>("file").unwrap().clone(),
            cfg_path: sub_matches.get_one::<PathBuf>("config").unwrap().clone(),
            root: sub_matches.get_one::<PathBuf>("root").unwrap().clone(),
            truncate: *sub_matches.get_one::<usize>("truncate").unwrap(),
            only_direct_deps: sub_matches.get_flag("only-direct-deps"),
        },
        Some(("test-config", sub_matches)) => CliCommand::TestConfig {
            test_cases_folder: sub_matches
                .get_one::<PathBuf>("test_cases_folder")
                .unwrap()
                .clone(),
            cfg_path: sub_matches.get_one::<PathBuf>("config").unwrap().clone(),
            fix_errors: sub_matches.get_flag("fix-errors"),
        },
        _ => CliCommand::Lint {
            root: matches.get_one::<PathBuf>("root").unwrap().clone(),
            cfg_path: matches.get_one::<PathBuf>("config").unwrap().clone(),
            allow_warnings: matches.get_flag("allow-warnings"),
        },
    }
}

pub fn get_cli_command() -> CliCommand {
    get_cli_cmd_from_matches(&get_clap_command().get_matches())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_cli_cmd_from_shell_string(shell_string: Vec<&str>) -> CliCommand {
        let cli = get_clap_command();
        let matches = cli.clone().try_get_matches_from(shell_string).unwrap();
        get_cli_cmd_from_matches(&matches)
    }

    #[test]
    fn test_lint_command_variations() {
        // Test default values
        assert_eq!(
            get_cli_cmd_from_shell_string(vec!["palinter", "--root", "."]),
            CliCommand::Lint {
                root: PathBuf::from("."),
                cfg_path: PathBuf::from("palinter.yaml"),
                allow_warnings: false,
            }
        );

        // Test with custom root directory
        assert_eq!(
            get_cli_cmd_from_shell_string(
                vec!["palinter", "--root", "src/project",]
            ),
            CliCommand::Lint {
                root: PathBuf::from("src/project"),
                cfg_path: PathBuf::from("palinter.yaml"),
                allow_warnings: false,
            }
        );

        // Test with all options combined
        assert_eq!(
            get_cli_cmd_from_shell_string(vec![
                "palinter",
                "--root",
                "src/project",
                "--config",
                "custom-config.yaml",
                "--allow-warnings",
            ]),
            CliCommand::Lint {
                root: PathBuf::from("src/project"),
                cfg_path: PathBuf::from("custom-config.yaml"),
                allow_warnings: true,
            }
        );
    }

    #[test]
    fn test_circular_deps_command_variations() {
        // Test with default config and root
        assert_eq!(
            get_cli_cmd_from_shell_string(vec![
                "palinter",
                "circular-deps",
                "src/app.ts",
            ]),
            CliCommand::CircularDeps {
                file_name: PathBuf::from("src/app.ts"),
                cfg_path: PathBuf::from("palinter.yaml"),
                root: PathBuf::from("."),
                truncate: 10,
                only_direct_deps: false,
            }
        );

        // Test with all options specified
        assert_eq!(
            get_cli_cmd_from_shell_string(vec![
                "palinter",
                "circular-deps",
                "file.ts",
                "--config",
                "palinter-2.yaml",
                "--root",
                "src/project",
                "--truncate",
                "5",
            ]),
            CliCommand::CircularDeps {
                file_name: PathBuf::from("file.ts"),
                cfg_path: PathBuf::from("palinter-2.yaml"),
                root: PathBuf::from("src/project"),
                truncate: 5,
                only_direct_deps: false,
            }
        );

        assert_eq!(
            get_cli_cmd_from_shell_string(vec![
                "palinter",
                "circular-deps",
                "file.ts",
                "--only-direct-deps",
            ]),
            CliCommand::CircularDeps {
                file_name: PathBuf::from("file.ts"),
                cfg_path: PathBuf::from("palinter.yaml"),
                root: PathBuf::from("."),
                truncate: 10,
                only_direct_deps: true,
            }
        );

        assert_eq!(
            get_cli_cmd_from_shell_string(vec![
                "palinter",
                "circular-deps",
                "file.ts",
                "-D",
            ]),
            CliCommand::CircularDeps {
                file_name: PathBuf::from("file.ts"),
                cfg_path: PathBuf::from("palinter.yaml"),
                root: PathBuf::from("."),
                truncate: 10,
                only_direct_deps: true,
            }
        );
    }

    #[test]
    fn test_test_config_command() {
        // Test with custom config
        assert_eq!(
            get_cli_cmd_from_shell_string(vec![
                "palinter",
                "test-config",
                "test_cases",
                "--config",
                "palinter-2.yaml",
            ]),
            CliCommand::TestConfig {
                test_cases_folder: PathBuf::from("test_cases"),
                cfg_path: PathBuf::from("palinter-2.yaml"),
                fix_errors: false,
            }
        );

        // Test with fix-errors flag
        assert_eq!(
            get_cli_cmd_from_shell_string(vec![
                "palinter",
                "test-config",
                "test_cases",
                "--fix-errors",
            ]),
            CliCommand::TestConfig {
                test_cases_folder: PathBuf::from("test_cases"),
                cfg_path: PathBuf::from("palinter.yaml"),
                fix_errors: true,
            }
        );
    }
}
