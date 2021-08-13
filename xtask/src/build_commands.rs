use std::{env, process::Command};

use anyhow::{anyhow, Context};

use crate::{Cmd, CmdResult, project_root};

pub static BUILD_CMD_NAMES: &[&str] = &["core", "wasm", "py", "c", "all"];
pub static BUILD_CMDS: &[Cmd] = &[
    cmd_build_core,
    cmd_build_wasm,
    cmd_build_py,
    cmd_build_c,
    cmd_build_all,
];

pub fn cmd_build_all() -> CmdResult<()> {
    cmd_build_core().with_context(|| "Building core failed")?;
    cmd_build_py().with_context(|| "Building py failed")?;
    cmd_build_c().with_context(|| "Building c? failed")?;
    cmd_build_wasm().with_context(|| "Building wasm failed")?;
    Ok(())
}

pub fn cmd_build_core() -> CmdResult<()> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let task = Command::new(cargo)
        .current_dir(project_root().join("cao-lang"))
        .args(&["build", "--release"])
        .spawn()
        .expect("Failed to spawn cargo task");

    let output = task.wait_with_output().unwrap();
    if !output.status.success() {
        return Err(anyhow!("Failed to build core"));
    }
    Ok(())
}

pub fn cmd_build_py() -> CmdResult<()> {
    todo!()
}

pub fn cmd_build_c() -> CmdResult<()> {
    todo!()
}

pub fn cmd_build_wasm() -> CmdResult<()> {
    todo!()
}
