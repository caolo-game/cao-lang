//! Custom test commands
//!
use std::process::Command;

use anyhow::{anyhow, Context};

use crate::{
    build_commands::{build_c_interface, configure_c_interface},
    project_root, CmdResult,
};

pub fn cmd_test_c() -> CmdResult<()> {
    configure_c_interface(&["-DCAOLO_ENABLE_TESTING=ON"])?;
    build_c_interface()?;

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
