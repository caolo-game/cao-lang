//! Custom build commands
//!
use std::{ffi::OsStr, io, process::Command};

use anyhow::{anyhow, Context};

use crate::{project_root, CmdResult};

pub fn cmd_build_c(args: &[&str]) -> CmdResult<()> {
    configure_c_interface(args)?;
    build_c_interface()?;
    Ok(())
}

pub fn configure_c_interface<T>(args: impl IntoIterator<Item = T>) -> CmdResult<()>
where
    T: AsRef<OsStr>,
{
    std::fs::create_dir(project_root().join("build")).unwrap_or_default();
    let task = Command::new("cmake")
        .arg("..")
        .args(args)
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
    if !task
        .wait_with_output()
        .expect("Failed to wait for the `cmake` command")
        .status
        .success()
    {
        return Err(anyhow!("CMake configure failed"));
    }
    Ok(())
}

pub fn build_c_interface() -> CmdResult<()> {
    let task = Command::new("cmake")
        .args(&["--build", ".", "--clean-first"])
        .current_dir(project_root().join("build"))
        .spawn()
        .with_context(|| "Spawning the cmake build task failed")?;

    if !task
        .wait_with_output()
        .expect("Failed to wait for the `cmake` command")
        .status
        .success()
    {
        return Err(anyhow!("CMake build failed"));
    }
    Ok(())
}
