use cao_lang::{compiler, prelude::CaoProgram};
use clap::{Arg, Command};

fn main() {
    let app = Command::new("Cao-Lang Disassembler").arg(
        Arg::new("json")
            .long("json")
            .help("Accept json encoded cao-lang program"),
    );

    let args = app.get_matches();
    if args.is_present("json") {
        let reader = std::io::BufReader::new(std::io::stdin().lock());
        let pl: CaoProgram = serde_json::from_reader(reader).expect("Failed to deserialize");
        let compiled = compiler::compile(pl, None).expect("Failed to compile");

        compiled.print_disassembly();
    } else {
        panic!("Missing format")
    }
}
