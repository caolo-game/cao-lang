mod cmd_build;
mod cmd_test;
mod cmd_version;

use std::path::{Path, PathBuf};

use clap::{App, Arg, SubCommand};

type CmdResult<T> = Result<T, anyhow::Error>;

fn main() {
    let app = App::new("Cao-Lang tasks")
        .subcommand(
            SubCommand::with_name("version-bump")
            .after_help("This command bumps the versions of all modules in the repository and generates a new changelog.")
            .arg(
                Arg::with_name("TARGET")
                    .takes_value(true)
                    .required(true)
                    .possible_values(&["major", "minor", "patch"])
                    .multiple(false),
            )
            .arg(
                Arg::with_name("tag")
                    .short("t")
                    .required(false)
                    .help("Also create a git tag after bumping the versions")
            ),
        )
        .subcommand(
            SubCommand::with_name("build").arg(
                Arg::with_name("TARGET")
                    .takes_value(true)
                    .required(true)
                    .possible_values(&["c"])
                    .multiple(false),
            )
            .arg(
            Arg::with_name("--")
            .help("Arguments to pass to cmake configure")
            .takes_value(true).required(false).multiple(true)
            )
        )
        .subcommand(
            SubCommand::with_name("test").arg(
                Arg::with_name("TARGET")
                    .takes_value(true)
                    .required(true)
                    .possible_values(&["c"])
                    .multiple(false),
            )
            .arg(
            Arg::with_name("--")
            .help("Arguments to pass to cmake configure")
            .takes_value(true).required(false).multiple(true)
            )
            ,
        );
    let args = app.get_matches();

    if let Some(subcmd) = args.subcommand_matches("version-bump") {
        if let Some(target) = subcmd.value_of("TARGET") {
            let res = if subcmd.is_present("tag") {
                cmd_version::cmd_create_tag(target)
            } else {
                cmd_version::cmd_bump_version(target).map(|_| ())
            };
            if let Err(e) = res {
                eprintln!("Version bump failed: {}", e);
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
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}
