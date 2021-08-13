mod build_commands;
mod test_commands;

use std::path::{Path, PathBuf};

use build_commands::{BUILD_CMDS, BUILD_CMD_NAMES};
use clap::{App, Arg, SubCommand, Values};
use test_commands::{TEST_CMDS, TEST_CMD_NAMES};

type CmdResult<T> = Result<T, anyhow::Error>;
type Cmd = fn() -> CmdResult<()>;

fn main() {
    assert!(BUILD_CMDS.len() == BUILD_CMD_NAMES.len());
    assert!(TEST_CMDS.len() == TEST_CMD_NAMES.len());

    let app = App::new("Cao-Lang tasks")
        .subcommand(
            SubCommand::with_name("build").arg(
                Arg::with_name("target")
                    .takes_value(true)
                    .possible_values(BUILD_CMD_NAMES)
                    .multiple(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("test").arg(
                Arg::with_name("target")
                    .takes_value(true)
                    .possible_values(TEST_CMD_NAMES)
                    .multiple(true),
            ),
        );
    let args = app.get_matches();

    if let Some(targets) = args
        .subcommand_matches("build")
        .and_then(|b| b.values_of("target"))
    {
        if let Err(e) = execute_subcommand(
            targets,
            BUILD_CMD_NAMES,
            BUILD_CMDS,
            build_commands::cmd_build_all,
        ) {
            eprintln!("Build command failed: {}", e);
        }
    }
    if let Some(targets) = args
        .subcommand_matches("test")
        .and_then(|b| b.values_of("target"))
    {
        if let Err(e) = execute_subcommand(
            targets,
            TEST_CMD_NAMES,
            TEST_CMDS,
            test_commands::cmd_test_all,
        ) {
            eprintln!("Test command failed: {}", e);
        }
    }
}

fn execute_subcommand(
    targets: Values,
    command_names: &[&str],
    commands: &[Cmd],
    all_cmd: Cmd,
) -> CmdResult<()> {
    debug_assert!(command_names.len() == commands.len());

    let all = targets.clone().any(|x| x == "all");
    if all {
        all_cmd()?;
    } else {
        for t in targets {
            let cmd = command_names
                .iter()
                .enumerate()
                .find_map(|(i, x)| (*x == t).then(|| i))
                .unwrap();
            commands[cmd]()?;
        }
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
