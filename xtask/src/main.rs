mod cmd_build;
mod cmd_test;
mod cmd_version;

use std::path::{Path, PathBuf};

use clap::{builder::PossibleValuesParser, Arg, Command};

type CmdResult<T> = Result<T, anyhow::Error>;

fn main() {
    let app = Command::new("Cao-Lang tasks")
        .subcommand(
            Command::new("version-bump")
            .after_help("This command bumps the versions of all modules in the repository and generates a new changelog.")
            .arg(
                Arg::new("TARGET")
                    .num_args(1)
                    .required(true)
                    .value_parser(PossibleValuesParser::new( ["major", "minor", "patch"]))
            )
            .arg(
                Arg::new("tag")
                    .num_args(0)
                    .short('t')
                    .required(false)
                    .help("Also create a git tag after bumping the versions")
            ),
        )
        .subcommand(
            Command::new("build").arg(
                Arg::new("TARGET")
                    .num_args(1)
                    .required(true)
                    .value_parser(PossibleValuesParser::new( ["c"]))
            )
            .arg(
            Arg::new("--")
            .help("Arguments to pass to cmake configure")
                    .num_args(..).required(false)
            )
        )
        .subcommand(
            Command::new("test").arg(
                Arg::new("TARGET")
                    .num_args(1)
                    .required(true)
                    .value_parser(PossibleValuesParser::new( ["c"]))
            )
            .arg(
            Arg::new("--")
            .help("Arguments to pass to cmake configure")
            .num_args(..).required(false)
            )
            ,
        );
    let args = app.get_matches();
    let mut code = 0;

    if let Some(subcmd) = args.subcommand_matches("version-bump") {
        if let Some(target) = subcmd.get_one::<String>("TARGET") {
            let res = if subcmd.get_flag("tag") {
                cmd_version::cmd_create_tag(target.as_str())
            } else {
                cmd_version::cmd_bump_version(target).map(|_| ())
            };
            if let Err(e) = res {
                eprintln!("Version bump failed: {}", e);
                code = 1;
            }
        }
    }
    if let Some(subcmd) = args.subcommand_matches("build") {
        if let Some(target) = subcmd.get_one::<String>("TARGET") {
            match target.as_str() {
                "c" => {
                    let args = subcmd
                        .get_many::<String>("--")
                        .unwrap_or_default()
                        .map(|x| x.as_str())
                        .collect::<Vec<_>>();
                    if let Err(e) = cmd_build::cmd_build_c(args.as_slice()) {
                        eprintln!("Build command failed: {}", e);
                        code = 2;
                    }
                }
                _ => unreachable!(),
            }
        }
    }
    if let Some(subcmd) = args.subcommand_matches("test") {
        if let Some(target) = subcmd.get_one::<String>("TARGET") {
            match target.as_str() {
                "c" => {
                    let args = subcmd
                        .get_many::<String>("--")
                        .unwrap_or_default()
                        .map(|x| x.as_str())
                        .collect::<Vec<_>>();
                    if let Err(e) = cmd_test::cmd_test_c(args.as_slice()) {
                        eprintln!("Test command failed: {}", e);
                        code = 3;
                    }
                }
                _ => unreachable!(),
            }
        }
    }
    std::process::exit(code);
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}
