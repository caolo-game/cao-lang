mod cmd_build;
mod cmd_test;
mod cmd_version;

use std::path::{Path, PathBuf};

use clap::{App, Arg, SubCommand, Values};

type CmdResult<T> = Result<T, anyhow::Error>;
type Cmd = fn() -> CmdResult<()>;

fn main() {
    let app = App::new("Cao-Lang tasks")
        .subcommand(
            SubCommand::with_name("version-bump")
            .after_help("This command bumps the versions of all modules in the repository and generates a new changelog.")
            .arg(
                Arg::with_name("target")
                    .takes_value(true)
                    .required(true)
                    .possible_values(&["major", "minor", "patch"])
                    .multiple(false),
            ),
        )
        .subcommand(
            SubCommand::with_name("build").arg(
                Arg::with_name("target")
                    .takes_value(true)
                    .required(true)
                    .possible_values(&["c"])
                    .min_values(1)
                    .multiple(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("test").arg(
                Arg::with_name("target")
                    .takes_value(true)
                    .required(true)
                    .possible_values(&["c"])
                    .min_values(1)
                    .multiple(true),
            ),
        );
    let args = app.get_matches();

    if let Some(target) = args
        .subcommand_matches("version-bump")
        .and_then(|b| b.value_of("target"))
    {
        if let Err(e) = cmd_version::cmd_bump_version(target) {
            eprintln!("Version bump failed: {}", e);
        }
    }
    if let Some(targets) = args
        .subcommand_matches("build")
        .and_then(|b| b.values_of("target"))
    {
        if let Err(e) = execute_subcommand(targets, &["c"], &[cmd_build::cmd_build_c]) {
            eprintln!("Build command failed: {}", e);
        }
    }
    if let Some(targets) = args
        .subcommand_matches("test")
        .and_then(|b| b.values_of("target"))
    {
        if let Err(e) = execute_subcommand(targets, &["c"], &[cmd_test::cmd_test_c]) {
            eprintln!("Test command failed: {}", e);
        }
    }
}

fn execute_subcommand(targets: Values, command_names: &[&str], commands: &[Cmd]) -> CmdResult<()> {
    debug_assert!(command_names.len() == commands.len());

    for t in targets {
        let cmd = command_names
            .iter()
            .enumerate()
            .find_map(|(i, x)| (*x == t).then(|| i))
            .unwrap();
        commands[cmd]()?;
    }
    Ok(())
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}
