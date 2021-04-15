//! CLI tool to compile cao-lang programs
//!
use cao_lang::{compiler::CompileOptions, prelude::*};
use clap::App;

use cao_lang::version::VERSION_STR;

fn main() {
    let _matches = App::new("cao-lang compiler")
        .version(VERSION_STR)
        .get_matches();

    let options = CompileOptions {};

    let cu: CaoIr = match serde_json::from_reader(std::io::stdin()) {
        Ok(cu) => cu,
        Err(err) => {
            eprintln!("Failed to parse compilation unit: {}", err);
            return;
        }
    };

    match compile(cu, Some(options)) {
        Ok(res) => {
            println!("{}", serde_json::to_string(&res).unwrap());
        }
        Err(err) => {
            eprintln!("Failed to compile: {}", err);
        }
    }
}
