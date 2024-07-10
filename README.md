# rbf

## RBF -- Rust BrainF***

RBF is a BrainF*** interpreter and soon to be compiler.

RBF can be installed via `cargo install rbf`. Alternatively, you may use the crate as a library
by adding it as a dependency e.g. `cargo add rbf`.

## CLI Usage

Run a program

```sh
rbf -p ./examples/hello_world
```

Run code directly

```sh
rbf -c ',>,<.>.'
```

Use the `-h` flag to see all options.

## Library Usage

```rust
use std::io::Write; // Bring in Write trait to flush terminal write buffer.
use console::Term;  // Use console crate for simple one-char input.

fn main() {
    let term = Term::stdout();           // Create console Term struct for single-char input.
    let mut stdout = std::io::stdout();  // Get stdout for flushing current buffer.

    // Create some instructions. This should print your usual hello world.
    let example_instructions = ">>+<--[[<++>->-->+++>+<<<]-->++++]<<.<<-.<<..+++.>.<<-.>.+++.------.>>-.<+.>>.";

    // Create a Program struct with the instructions.
    let mut prgm = rbf::Program::from_string(example_instructions);

    // Create input and output for the BF interpreter.
    let input = rbf::BasicInput::new();
    let mut output = rbf::BasicOutput::new();

    // Execute the instructions and print if we get an error.
    // We define closures to tell the interpreter how to handle input and output.
    // `rbf` provides basic io structs to handle one-char input and output, which we use for
    // our input and output closures.
    match prgm.execute(|| input.input_char(), |c| output.print_char(c)) {
        Ok(()) => println!("\nProgram finished."),
        Err(e) => eprintln!("\n{}", e),
    };
}
```

Input and output is handled by the closures you define. You could capture output and read
from preset or procedural input e.g.

```rust
let mut output = String::new();

let charin = || 'a'; // always read input as 'a'
let charout = |c| output.push(c);
```
