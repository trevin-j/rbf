use std::fs;
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// BF file to interpret.
    #[arg(short, long, value_name = "FILE")]
    program: Option<PathBuf>,

    /// Raw BF string to interpret.
    #[arg(short, long, value_name = "CODE")]
    code: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let mut prgm: rbf::Program;

    if let Some(program) = cli.program.as_deref() {
        let program_contents: String;

        match fs::read_to_string(program) {
            Ok(program) => program_contents = program,
            Err(e) => {
                println!("Error reading from file: {}", e);
                return;
            }
        }
        prgm = rbf::Program::from_string(&program_contents);
    } else if let Some(code) = cli.code.as_deref() {
        prgm = rbf::Program::from_string(code);
    } else {
        println!("Must pass code via code or program argument.");
        return;
    }

    let input = rbf::BasicInput::new();
    let mut output = rbf::BasicOutput::new();

    match prgm.execute(|| input.input_char(), |c| output.print_char(c)) {
        Ok(()) => println!("\nProgram finished."),
        Err(e) => eprintln!("\n{}", e),
    };
}
