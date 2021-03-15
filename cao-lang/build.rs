use serde::Deserialize;
use std::fs;
use std::io::Read;
use std::{env, path::Path};

#[derive(Deserialize, Debug)]
struct Manifest {
    package: Package,
}

#[derive(Deserialize, Debug)]
struct Package {
    version: String,
}

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("cao_lang_version.rs");

    let mut mf = std::fs::OpenOptions::new()
        .read(true)
        .open("Cargo.toml")
        .expect("Failed to open the manifest file");

    let mut manifest = String::with_capacity(1024);
    mf.read_to_string(&mut manifest)
        .expect("Failed to read manifest");

    let conf: Manifest = toml::from_str(manifest.as_str()).expect("Failed to parse manifest");
    let version = semver::Version::parse(conf.package.version.as_str())
        .expect("Crate version wasn't valid semver");

    fs::write(
        &dest_path,
        format!(
            r#"
pub const VERSION_STR: &str = "{}";
pub const MAJOR: u8 = {};
pub const MINOR: u8 = {};
pub const PATCH: u16 = {};
"#,
            conf.package.version, version.major, version.minor, version.patch,
        ),
    )
    .expect("Failed to write version file");
}
