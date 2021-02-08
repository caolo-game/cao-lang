//! CLI tool to compile cao-lang programs
//!
use cao_lang::{compiler::CompileOptions, prelude::*};
use clap::{App, Arg};

use cao_lang::version::VERSION_STR;

fn main() {
    let matches = App::new("cao-lang compiler")
        .version(VERSION_STR)
        .arg(
            Arg::new("breadcrumbs")
                .short('b')
                .long("breadcrumbs")
                .about("Insert breadcrumbs into the program? Default: true")
                .takes_value(true)
                .required(false),
        )
        .get_matches();

    let options = CompileOptions {
        breadcrumbs: matches
            .value_of("breadcrumbs")
            .and_then(|s| s.parse().ok())
            .unwrap_or(true),
    };

    let cu: CompilationUnit = match serde_json::from_reader(std::io::stdin()) {
        Ok(cu) => cu,
        Err(err) => {
            eprintln!("Failed to parse compilation unit: {}", err);
            return;
        }
    };

    match compile(None, cu, Some(options)) {
        Ok(res) => {
            println!("{}", serde_json::to_string(&res).unwrap());
        }
        Err(err) => {
            eprintln!("Failed to compile: {}", err);
        }
    }
}
