mod cmd_build;
mod cmd_test;
mod cmd_version;

use std::path::{Path, PathBuf};

use clap::{App, Arg};

type CmdResult<T> = Result<T, anyhow::Error>;

fn main() {
    let app = App::new("Cao-Lang tasks")
        .subcommand(
            App::new("version-bump")
            .after_help("This command bumps the versions of all modules in the repository and generates a new changelog.")
            .arg(
                Arg::new("TARGET")
                    .takes_value(true)
                    .required(true)
                    .possible_values(&["major", "minor", "patch"])
                    .multiple_occurrences(false),
            )
            .arg(
                Arg::new("tag")
                    .short('t')
                    .required(false)
                    .help("Also create a git tag after bumping the versions")
            ),
        )
        .subcommand(
            App::new("build").arg(
                Arg::new("TARGET")
                    .takes_value(true)
                    .required(true)
                    .possible_values(&["c"])
                    .multiple_occurrences(false),
            )
            .arg(
            Arg::new("--")
            .help("Arguments to pass to cmake configure")
            .takes_value(true).required(false).multiple_occurrences(true)
            )
        )
        .subcommand(
            App::new("test").arg(
                Arg::new("TARGET")
                    .takes_value(true)
                    .required(true)
                    .possible_values(&["c"])
                    .multiple_occurrences(false),
            )
            .arg(
            Arg::new("--")
            .help("Arguments to pass to cmake configure")
            .takes_value(true).required(false).multiple_occurrences(true)
            )
            ,
        );
    let args = app.get_matches();
    let mut code = 0;

    if let Some(subcmd) = args.subcommand_matches("version-bump") {
        if let Some(target) = subcmd.value_of("TARGET") {
            let res = if subcmd.is_present("tag") {
                cmd_version::cmd_create_tag(target)
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
        if let Some(target) = subcmd.value_of("TARGET") {
            match target {
                "c" => {
                    let args = subcmd
                        .values_of("--")
                        .unwrap_or_default()
                        .into_iter()
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
        if let Some(target) = subcmd.value_of("TARGET") {
            match target {
                "c" => {
                    let args = subcmd
                        .values_of("--")
                        .unwrap_or_default()
                        .into_iter()
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
