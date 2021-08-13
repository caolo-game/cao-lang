use std::{env, io, process::Command};

use anyhow::{anyhow, Context};

use crate::{project_root, Cmd, CmdResult};

pub static TEST_CMD_NAMES: &[&str] = &["py", "core", "c", "all"];
pub static TEST_CMDS: &[Cmd] = &[cmd_test_py, cmd_test_core, cmd_test_c, cmd_test_all];

pub fn cmd_test_all() -> CmdResult<()> {
    cmd_test_core().with_context(|| "Testing core failed")?;
    cmd_test_py().with_context(|| "Testing python failed")?;
    Ok(())
}

pub fn cmd_test_core() -> CmdResult<()> {
    let test_args: &[&[&str]] = &[
        &["check-all-features"],
        &["test", "--tests", "--all", "--benches"],
        &["test", "--doc"],
    ];

    for args in test_args {
        println!("Running `cargo with args {:?}", args);
        let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
        let task = Command::new(cargo)
            .current_dir(project_root().join("cao-lang"))
            .args(*args)
            .spawn()
            .expect("Failed to spawn cargo task");

        let output = task.wait_with_output().unwrap();
        if !output.status.success() {
            return Err(anyhow!("Test {:?} failed", args));
        }
    }

    Ok(())
}

pub fn cmd_test_py() -> CmdResult<()> {
    let task = Command::new("tox").current_dir(project_root()).spawn();

    let task = match task {
        Ok(o) => o,
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => {
                return Err(anyhow!(
                    "`tox` command can not be found!\nTry installing it with `pip install tox`"
                ))
            }
            _ => {
                return Err(err).with_context(|| "Tox failed");
            }
        },
    };

    let output = task
        .wait_with_output()
        .expect("Failed to wait for the `tox` command");

    if !output.status.success() {
        return Err(anyhow!("Failed to build the python wrapper"));
    }

    Ok(())
}

pub fn cmd_test_c() -> CmdResult<()> {
    std::fs::create_dir(project_root().join("build")).unwrap_or_default();

    // configure
    //
    let task = Command::new("cmake")
        .args(&["..", "-DCAOLO_ENABLE_TESTING=ON"])
        .current_dir(project_root().join("build"))
        .spawn();

    let task = match task {
        Ok(o) => o,
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => return Err(anyhow!("`cmake` command can not be found!")),
            _ => {
                return Err(err).with_context(|| "cmake configure failed");
            }
        },
    };
    let output = task
        .wait_with_output()
        .expect("Failed to wait for the `cmake` command");
    if !output.status.success() {
        return Err(anyhow!("CMake configure failed"));
    }

    // build
    //
    let task = Command::new("cmake")
        .args(&["--build", ".", "--clean-first"])
        .current_dir(project_root().join("build"))
        .spawn()
        .with_context(|| "Spawning the cmake build task failed")?;

    let output = task
        .wait_with_output()
        .expect("Failed to wait for the `cmake` command");
    if !output.status.success() {
        return Err(anyhow!("CMake build failed"));
    }

    // run the tests
    //
    let task = Command::new("ctest")
        .args(&["--output-on-failure"])
        .current_dir(project_root().join("build"))
        .spawn()
        .with_context(|| "Spawning the ctest task failed")?;

    let output = task
        .wait_with_output()
        .expect("Failed to wait for the `ctest` command");
    if !output.status.success() {
        return Err(anyhow!("CTest failed"));
    }
    Ok(())
}
