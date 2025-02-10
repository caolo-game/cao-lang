use std::process::Command;

use crate::{project_root, CmdResult};
use anyhow::Context;
use semver::Version;

pub fn cmd_bump_version(target: &str) -> CmdResult<String> {
    assert_git_not_dirty()
        .with_context(|| "Please commit your changes before creating a new version")?;
    let new_version =
        bump_cargo_manifest_version(project_root().join("cao-lang").join("Cargo.toml"), target)
            .with_context(|| "Failed to bump core version")?;
    bump_cargo_manifest_version(project_root().join("wasm").join("Cargo.toml"), target)
        .with_context(|| "Failed to bump wasm version")?;
    bump_cargo_manifest_version(project_root().join("py").join("Cargo.toml"), target)
        .with_context(|| "Failed to bump python version")?;

    let new_version = format!("v{}", new_version);
    make_changelog(&new_version)?;

    println!("New core version: {}", new_version);
    Ok(new_version)
}

pub fn cmd_create_tag(version_target: &str) -> CmdResult<()> {
    let new_version = cmd_bump_version(version_target)?;

    commit_bump(&new_version)?;
    git_tag(&new_version)?;

    println!("Version bump successful. Push the new version: `git push --tags`");

    Ok(())
}

fn assert_git_not_dirty() -> CmdResult<()> {
    let task = Command::new("git")
        .args(["diff", "--stat"])
        .current_dir(project_root())
        .output()
        .with_context(|| "Failed to spawn git")?;

    if !task.stdout.is_empty() {
        return Err(anyhow::anyhow!("Git repository is dirty"));
    }

    Ok(())
}

fn commit_bump(version: &str) -> CmdResult<()> {
    let msg = format!("Bump version to - {version}");
    let task = Command::new("git")
        .args(["commit", "-am", msg.as_str()])
        .current_dir(project_root())
        .spawn()
        .with_context(|| "Failed to spawn git")?;

    if !task
        .wait_with_output()
        .expect("Failed to wait for git")
        .status
        .success()
    {
        return Err(anyhow::anyhow!("git commit failed"));
    }
    Ok(())
}

fn git_tag(tag: &str) -> CmdResult<()> {
    let task = Command::new("git")
        .args(["tag", tag])
        .current_dir(project_root())
        .spawn()
        .with_context(|| "Failed to spawn git")?;

    if !task
        .wait_with_output()
        .expect("Failed to wait for git")
        .status
        .success()
    {
        return Err(anyhow::anyhow!("git tag failed"));
    }
    Ok(())
}

fn make_changelog(tag: &str) -> CmdResult<()> {
    let task = Command::new("git-cliff")
        .args(["-o", "CHANGELOG.md", "--tag", tag])
        .current_dir(project_root())
        .spawn()
        .with_context(|| "Failed to spawn git cliff")?;

    if !task
        .wait_with_output()
        .expect("Failed to wait for git-cliff")
        .status
        .success()
    {
        return Err(anyhow::anyhow!("Git cliff failed"));
    }
    Ok(())
}

fn bump_cargo_manifest_version(
    manifest_path: std::path::PathBuf,
    target: &str,
) -> CmdResult<Version> {
    let mut core_manifest_str =
        std::fs::read_to_string(&manifest_path).with_context(|| "Failed to read core manifest")?;

    core_manifest_str = core_manifest_str.replace("\r\n", "\n");
    let mut core_manifest: toml::Value =
        toml::from_str(&core_manifest_str).with_context(|| "Failed to parse core manifest")?;

    let package = core_manifest
        .get_mut("package")
        .with_context(|| "Failed to get package section of core manifest")?;
    let current_version = package
        .get("version")
        .with_context(|| "Failed to get version str")?;

    let mut version = match current_version.as_str() {
        Some(vstr) => semver::Version::parse(vstr).with_context(|| "Failed to parse version")?,
        None => {
            return Err(anyhow::anyhow!("Expected version to be a string"));
        }
    };

    match target {
        "major" => {
            version.major += 1;
            version.minor = 0;
            version.patch = 0;
        }
        "minor" => {
            version.minor += 1;
            version.patch = 0;
        }
        "patch" => {
            version.patch += 1;
        }
        _ => unreachable!(),
    };

    package.as_table_mut().unwrap().insert(
        "version".to_string(),
        toml::Value::String(version.to_string()),
    );

    let content = toml::to_string(&core_manifest).with_context(|| "Failed to serialize toml")?;

    std::fs::write(manifest_path, content).with_context(|| "Failed to write manifest")?;

    Ok(version)
}
