# rbf

## RBF -- Rust BrainF***

RBF is a BrainF*** interpreter.

RBF is not on crates.io due to being a simple fun project to learn rust. It's not for any
practical use case. Also another project on crates.io uses the name RBF I believe and this
small rust project is not related to that one in any way.

Despite not being a practical application, documentation is decent. Check out the documentation
for using this project as a binary or library with `cargo doc --open`.

In addition to documentation, there are some tests. Run with `cargo test`.

## Optimizations

One neat feature is that I included an optimization for the BF interpreter. Passing the -o flag
enables optimization. This works by optimizing the internal BF instructions before executing
them. The only optimization currently implemented is instruction collapsing, where repeated
operations are collapsed into a single operation. Additional optimizations were originally
planned but I decided they wouldn't have a big enough impact to be worth implementing. With
only the code collapse optimization, it still yields a whopping ~60% performance increase when
tested with the examples/mandelbrot.bf example!

For better performance, build/run in release mode. Add `--release` before the `--` in the
`cargo run` command.

### Additional possible optimizations

I initially planned to add optimizations for common BF algorithms found [here](https://esolangs.org/wiki/Brainfuck_algorithms).
However, like I mentioned above, it would likely not be worth my time for this project. The
optimizations would work by converting a recognized BF algorithm into a single (or few)
operation e.g. multiplication.

## Install

Like mentioned, not on crates.io but can be installed with `git clone
https://github.com/trevin-j/rbf`.

## CLI Usage

Run a program

```sh
cargo run -- -p ./examples/hello_world
```

Run code directly

```sh
cargo run -- -c ',>,<.>.'
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
