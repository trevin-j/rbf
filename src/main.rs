use std::fs;
use std::path::PathBuf;
use std::time::Instant;

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

    /// Time how long it takes to execute a BF program. If -r, time includes execution of all
    /// repetitions.
    #[arg(short, long)]
    benchmark: bool,

    /// Repeat the program NUMBER times. Useful for benchmarking.
    #[arg(short, long, value_name = "NUMBER", default_value_t = 1)]
    repititions: usize,

    /// Use blank IO, useful for benchmarking.
    #[arg(long)]
    blank_io: bool,

    /// Run internal optimization on the BF code.
    #[arg(short, long)]
    optimize: bool,
}

fn main() {
    let cli = Cli::parse();

    let mut prgm: rbf::Program;
    let mut instructions: rbf::Instructions;

    if let Some(program) = cli.program.as_deref() {
        let program_contents: String;

        match fs::read_to_string(program) {
            Ok(program) => program_contents = program,
            Err(e) => {
                println!("Error reading from file: {}", e);
                return;
            }
        }

        instructions = rbf::Instructions::from_string(&program_contents);
    } else if let Some(code) = cli.code.as_deref() {
        instructions = rbf::Instructions::from_string(code);
    } else {
        println!("Must pass code via code or program argument.");
        return;
    }

    if cli.optimize {
        instructions.optimize();
    }

    prgm = rbf::Program::new(instructions);

    let input = rbf::BasicInput::new();
    let mut output = rbf::BasicOutput::new();

    let mut input_fn: Box<dyn FnMut() -> char>;
    let mut output_fn: Box<dyn FnMut(char)>;

    if cli.blank_io {
        input_fn = Box::new(|| input.blank());
        output_fn = Box::new(|c| output.blank(c));
    } else {
        input_fn = Box::new(|| input.input_char());
        output_fn = Box::new(|c| output.print_char(c));
    }

    let before = Instant::now();

    for _ in 0..cli.repititions {
        prgm.reset();
        match prgm.execute(|| input_fn(), |c| output_fn(c)) {
            Ok(()) => {}
            Err(e) => eprintln!("\n{}", e),
        };
    }

    if cli.repititions > 1 {
        println!("\nFinished executing {} times.", cli.repititions);
    } else {
        println!("\nFinished program.");
    }

    if cli.benchmark {
        let elapsed = before.elapsed();
        println!("Took: {:.2?}", elapsed);
    }
}
