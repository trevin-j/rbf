//! # RBF -- Rust BrainF***
//!
//! RBF is a BrainF*** interpreter and soon to be compiler.
//!
//! RBF can be installed via `cargo install rbf`. Alternatively, you may use the crate as a library
//! by adding it as a dependency e.g. `cargo add rbf`.
//!
//! # CLI Usage
//!
//! Run a program
//!
//! ```sh
//! rbf -p ./examples/hello_world
//! ```
//!
//! Run code directly
//!
//! ```sh
//! rbf -c ',>,<.>.'
//! ```
//!
//! Use the `-h` flag to see all options.
//!
//! # Library Usage
//!
//! ```rust
//! use std::io::Write; // Bring in Write trait to flush terminal write buffer.
//! use console::Term;  // Use console crate for simple one-char input.
//!
//! fn main() {
//!     let term = Term::stdout();           // Create console Term struct for single-char input.
//!     let mut stdout = std::io::stdout();  // Get stdout for flushing current buffer.
//!
//!     // Create some instructions. This should print your usual hello world.
//!     let example_instructions = ">>+<--[[<++>->-->+++>+<<<]-->++++]<<.<<-.<<..+++.>.<<-.>.+++.------.>>-.<+.>>.";
//!
//!     // Create a Program struct with the instructions.
//!     let mut prgm = rbf::Program::from_string(example_instructions);
//!
//!     // Create input and output for the BF interpreter.
//!     let input = rbf::BasicInput::new();
//!     let mut output = rbf::BasicOutput::new();
//!
//!     // Execute the instructions and print if we get an error.
//!     // We define closures to tell the interpreter how to handle input and output.
//!     // `rbf` provides basic io structs to handle one-char input and output, which we use for
//!     // our input and output closures.
//!     match prgm.execute(|| input.input_char(), |c| output.print_char(c)) {
//!         Ok(()) => println!("\nProgram finished."),
//!         Err(e) => eprintln!("\n{}", e),
//!     };
//! }
//! ```
//!
//! Input and output is handled by the closures you define. You could capture output and read
//! from preset or procedural input e.g.
//!
//! ```rust
//! let mut output = String::new();
//!
//! let charin = || 'a'; // always read input as 'a'
//! let charout = |c| output.push(c);
//! ```

#![warn(missing_docs)]

use std::io::Write;

use console::Term;

pub mod errors;
use errors::{BFError, BFErrorKind};

type Result<T> = std::result::Result<T, BFError>;

/// Represents a BF instruction.
///
/// The `isize` values for MvPtr and MvValue are for future optimization purposes, representing
/// multiple of a single command.
#[derive(Debug, PartialEq, Clone)]
enum Instruct {
    MvPtr(isize),
    MvValue(isize),
    Output,
    Input,
    OpenLoop,
    CloseLoop,
}

/// Holds each converted BF Instruct in a Vec to be interpretted.
///
/// `rbf::Instructions` contains a `Vec<Instruct>`. The `rbf::Instruct` enum, which is private, is an
/// enum representing a single BF instruction. The Instructions struct implements methods to
/// convert a string of BF instructions into the `Vec<Instruct>` that can be interpretted by the
/// Program struct. `rbf::Instructions` will be used to optimize the code as well, such as combining
/// multiple of the same instruction, and finding patterns such as multiplication loops.
///
/// # Examples
///
/// ```rust
/// # use rbf::{Instructions, Program};
/// let instructions = Instructions::from_string(",>,<.>.");
/// let prgm = Program::new(instructions);
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct Instructions(Vec<Instruct>);

impl Instructions {
    /// Convert a string slice of commands into an Instructions struct containing the converted instructions.
    ///
    /// # Arguments
    ///
    /// * `commands` - A string slice holding the raw BrainF*** instructions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rbf::Instructions;
    /// let instructions = Instructions::from_string(",>,<.>.");
    /// ```
    pub fn from_string(commands: &str) -> Instructions {
        Instructions(commands.chars().fold(Vec::new(), |mut acc, c| {
            let instruction = match c {
                '>' => Some(Instruct::MvPtr(1)),
                '<' => Some(Instruct::MvPtr(-1)),
                '-' => Some(Instruct::MvValue(-1)),
                '+' => Some(Instruct::MvValue(1)),
                '.' => Some(Instruct::Output),
                ',' => Some(Instruct::Input),
                '[' => Some(Instruct::OpenLoop),
                ']' => Some(Instruct::CloseLoop),
                _ => None, // Anything other than valid commands is simply a comment! :)
            };
            if let Some(i) = instruction {
                acc.push(i);
            }
            acc
        }))
    }
}

/// Holds the BF program's functionality.
///
/// It contains methods for execution such as stepwise executing and full-program executing.
///
/// # Examples
///
/// Create Program struct directly from a string of BF code, and execute it.
///
/// ```rust
/// # use rbf::*;
/// let mut prgm = Program::from_string(",>,<.>.");
///
/// # let basic_input = BasicInput::new();
/// # let mut basic_output = BasicOutput::new();
/// # let input_closure = || basic_input.input_char();
/// # let output_closure = |c| basic_output.print_char(c);
/// prgm.execute(input_closure, output_closure).expect("Error in BF instructions.");
/// ```
#[derive(Debug, PartialEq)]
pub struct Program {
    /// Instructions to execute.
    instructions: Instructions,
    /// Pointer to where in the instructions we are currently looking.
    instruction_ptr: usize,

    /// Program's memory -- Vec of cells in the ascii range of numbers.
    cells: Vec<u8>,
    /// Current location in memory.
    cell_ptr: usize,

    /// Simple var to manage loops.
    loop_stack: Vec<usize>,
}

impl Program {
    /// Clear and reset the program state.
    ///
    /// Clears the cells, instruction pointer, cell pointer, and loop stack. Subsequently
    /// calling `Program::execute()` or `Program::step()` will begin the program from the
    /// beginning.
    pub fn reset(&mut self) {
        self.instruction_ptr = 0;
        self.cells.clear();
        self.cell_ptr = 0;
        self.loop_stack.clear();
    }

    /// Create a new program struct.
    ///
    /// This constructor requires the instructions to already be represented by an `Instructions`
    /// struct.
    pub fn new(instructions: Instructions) -> Program {
        Program {
            instructions,
            instruction_ptr: 0,
            cells: vec![],
            cell_ptr: 0,
            loop_stack: vec![],
        }
    }

    /// Create a new program directly from a string of BF code.
    ///
    /// This method is a wrapper of the Program::new() method, creating a new Instructions
    /// struct from the instructions string first.
    pub fn from_string(instructions: &str) -> Program {
        Self::new(Instructions::from_string(instructions))
    }

    /// Execute the entire BF program.
    ///
    /// Step-by-step interprets the entire BF program according to its Instructions.
    ///
    /// # Errors
    ///
    /// Will return an error if the instructions are invalid. These errors are runtime BF
    /// errors.
    pub fn execute<Fin, Fout>(&mut self, mut input: Fin, mut output: Fout) -> Result<()>
    where
        Fin: FnMut() -> char,
        Fout: FnMut(char),
    {
        loop {
            if self.done()? {
                break;
            }
            self.step(|| input(), |c| output(c))?;
        }

        Ok(())
    }

    /// Execute the next step in the BF program.
    pub fn step<Fin, Fout>(&mut self, input: Fin, output: Fout) -> Result<()>
    where
        Fin: FnOnce() -> char,
        Fout: FnOnce(char),
    {
        // Make sure cells length is good so any possible operations we do work.
        self.validate_cells_length();

        let instruction = match self.instructions.0.get(self.instruction_ptr) {
            Some(i) => i,
            None => {
                return Err(BFError {
                    kind: BFErrorKind::InstructionBoundsError,
                })
            }
        };

        // println!( // Dirty debugging
        //     "i: {:?}; iptr: {}; cptr: {}; cv: {};",
        //     instruction, self.instruction_ptr, self.cell_ptr, self.cells[self.cell_ptr]
        // );

        match *instruction {
            Instruct::MvPtr(n) => self.move_cell_pointer(&n)?,
            Instruct::MvValue(n) => self.move_cell_value(&n),
            Instruct::Input => self.input_cell(input)?,
            Instruct::Output => self.output_cell(output),
            Instruct::OpenLoop => self.open_loop()?,
            Instruct::CloseLoop => self.close_loop()?,
        }

        self.instruction_ptr += 1;

        Ok(())
    }

    /// Check if the program has finished executing.
    pub fn done(&self) -> Result<bool> {
        if self.instruction_ptr >= self.instructions.0.len() {
            if self.loop_stack.len() > 0 {
                Err(BFError {
                    kind: BFErrorKind::MissingClose,
                })
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    /// Move the cell pointer either right or left. BF instructions ">" and "<" respectively.
    ///
    /// Note that it takes an amount. If there are repeating ">" or "<" instructions, rather
    /// than move multiple times in a row, it can be optimized and moved only once, x spaces.
    fn move_cell_pointer(&mut self, amount: &isize) -> Result<()> {
        self.cell_ptr = match self.cell_ptr.checked_add_signed(*amount) {
            Some(val) => val,
            None => {
                return Err(BFError {
                    kind: BFErrorKind::CellBoundsError,
                })
            }
        };

        Ok(())
    }

    /// Check the cells length and make sure it's long enough such that cell_ptr is a valid index.
    fn validate_cells_length(&mut self) {
        while self.cells.len() <= self.cell_ptr {
            self.cells.push(0);
        }
    }

    /// Increment/decrement current cell value by `amount`.
    ///
    /// Multiple subsequent calls to this can be replaced by a single call with the sum in
    /// order to optimize.
    fn move_cell_value(&mut self, amount: &isize) {
        self.cells[self.cell_ptr] = self.cells[self.cell_ptr].wrapping_add_signed(*amount as i8);
    }

    /// Using the input closure, retrieve a character into the cells at cell ptr.
    fn input_cell<F>(&mut self, input: F) -> Result<()>
    where
        F: FnOnce() -> char,
    {
        let in_char = input();
        let in_byte = in_char as u32;

        // Gotta check to make sure it's only 8 bit int
        if in_byte < 256 {
            self.cells[self.cell_ptr] = in_byte as u8;
            Ok(())
        } else {
            Err(BFError {
                kind: BFErrorKind::InvalidInput,
            })
        }
    }

    /// Output a character at current cell into the output closure.
    fn output_cell<F>(&self, output: F)
    where
        F: FnOnce(char),
    {
        output(self.cells[self.cell_ptr] as char);
    }

    /// Handle the open loop instructions, `[`.
    fn open_loop(&mut self) -> Result<()> {
        if self.cells[self.cell_ptr] > 0 {
            self.loop_stack.push(self.instruction_ptr);
        } else {
            self.move_to_closed_loop()?;
        }

        Ok(())
    }

    /// Handle the close loop instruction, ']'.
    fn close_loop(&mut self) -> Result<()> {
        self.instruction_ptr = match self.loop_stack.pop() {
            Some(n) => n,
            None => {
                return Err(BFError {
                    kind: BFErrorKind::MissingOpen,
                });
            }
        } - 1;
        Ok(())
    }

    /// Find the associated close loop to our current open loop and go there.
    fn move_to_closed_loop(&mut self) -> Result<()> {
        let mut loopstack: Vec<usize> = vec![];
        let mut current_instruction = self.instruction_ptr + 1; // We don't want to add
                                                                // current open loop to stack
        loop {
            let instruction = match self.instructions.0.get(current_instruction) {
                Some(i) => i,
                None => {
                    return Err(BFError {
                        kind: BFErrorKind::MissingClose,
                    });
                }
            };

            match instruction {
                Instruct::OpenLoop => loopstack.push(current_instruction),
                Instruct::CloseLoop => {
                    if loopstack.pop().is_none() {
                        self.instruction_ptr = current_instruction;
                        return Ok(());
                    }
                }
                _ => (),
            }

            current_instruction += 1;
        }
    }
}

/// Basic input operation for BF.
///
/// Provides a method that can be used for the input of the BF program.
///
/// # Examples
///
/// ```rust
/// # use rbf::*;
/// let basic_input = BasicInput::new();
///
/// // Read single char from terminal.
/// let c = basic_input.input_char();
/// ```
pub struct BasicInput {
    term: Term,
}

/// Basic output operation for BF.
///
/// Provides a method for output of the BF program.
///
/// # Examples
///
/// ```rust
/// # use rbf::*;
/// let mut basic_output = BasicOutput::new();
///
/// // Output single char to terminal.
/// basic_output.print_char('a');
/// ```
pub struct BasicOutput {
    stdout: std::io::Stdout,
}

impl BasicInput {
    /// Create new BasicInput struct.
    pub fn new() -> Self {
        Self {
            term: Term::stdout(),
        }
    }

    /// Blank input, handy when benchmarking and input doesn't matter.
    pub fn blank(&self) -> char {
        ' '
    }

    /// Input single char from terminal.
    ///
    /// If the terminal is not an interactive terminal, the terminal from the `console` crate
    /// returns an error from `read_char()`. In this situation, this function will return a char
    /// with ascii value of 0.
    pub fn input_char(&self) -> char {
        match self.term.read_char() {
            Ok(c) => c,
            Err(_) => 0u8 as char,
        }
    }
}

impl BasicOutput {
    /// Create new BasicOutput struct.
    pub fn new() -> Self {
        Self {
            stdout: std::io::stdout(),
        }
    }

    /// Blank output, handy when benchmarking and output doesn't matter.
    pub fn blank(&self, _: char) {}

    /// Print single char to terminal.
    pub fn print_char(&mut self, c: char) {
        print!("{}", c);
        self.stdout.flush().expect("Error flushing output");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Execute program with blanks without boilerplate.
    ///
    /// Will panic in the case of a BF error.
    fn blank_execute_prgm(prgm: &mut Program) -> Result<()> {
        let input = BasicInput::new();
        let output = BasicOutput::new();

        prgm.execute(|| input.blank(), |c| output.blank(c))
    }

    #[test]
    fn str_to_instructions() {
        let instructions_str = "+-<>s[]comment,."; // the 's' and the word 'comment' are
                                                   // comments and should be ignored in the output
        let instructions = Instructions::from_string(instructions_str);
        assert_eq!(
            instructions,
            Instructions(vec![
                Instruct::MvValue(1),
                Instruct::MvValue(-1),
                Instruct::MvPtr(-1),
                Instruct::MvPtr(1),
                Instruct::OpenLoop,
                Instruct::CloseLoop,
                Instruct::Input,
                Instruct::Output,
            ])
        );
    }

    #[test]
    fn create_program() {
        let instructions = Instructions::from_string("+-><[],.");
        let new_program = Program::new(instructions.clone());
        let custom_program = Program {
            instructions,
            instruction_ptr: 0,
            cell_ptr: 0,
            cells: vec![],
            loop_stack: vec![],
        };

        assert_eq!(new_program, custom_program);
    }

    #[test]
    fn instruction_bounds_error() {
        // Should error if trying to access instruction out of bounds e.g. stepping after
        // program has already finished.
        let mut prgm = Program::from_string("+-><[],.");

        blank_execute_prgm(&mut prgm).unwrap();

        let input = BasicInput::new();
        let output = BasicOutput::new();

        let result = prgm
            .step(|| input.blank(), |c| output.blank(c))
            .map_err(|e| e.kind);
        let expected = Err(BFErrorKind::InstructionBoundsError);
        assert_eq!(result, expected);
    }

    #[test]
    fn cell_bounds_error() {
        // Should error if we try to access a cell outside of bounds in BF.
        let mut prgm = Program::from_string("<+");

        let result = blank_execute_prgm(&mut prgm).map_err(|e| e.kind);
        let expected = Err(BFErrorKind::CellBoundsError);
        assert_eq!(result, expected);
    }

    #[test]
    fn invalid_input() {
        // If the BF program receives invalid input e.g. char values larger than 255.
        let mut prgm = Program::from_string(",");

        // Try passing too big a char as input.
        let result = prgm.execute(|| '\u{10FFFF}', |_| ()).map_err(|e| e.kind);
        let expected = Err(BFErrorKind::InvalidInput);
        assert_eq!(result, expected);
    }

    #[test]
    fn missing_open_bracket() {
        let mut prgm = Program::from_string("++>+++>+.<.]-<+++");

        let result = blank_execute_prgm(&mut prgm).map_err(|e| e.kind);
        let expected = Err(BFErrorKind::MissingOpen);
        assert_eq!(result, expected);
    }

    #[test]
    fn missing_close_bracket() {
        // Situation where it wouldn't yet skip to closing bracket
        let mut prgm = Program::from_string("++>+++>+.<.[-<+++");
        // Situation where it would
        let mut prgm2 = Program::from_string("++>+++>+.<.>>>[-<+++");

        let result = blank_execute_prgm(&mut prgm).map_err(|e| e.kind);
        let result2 = blank_execute_prgm(&mut prgm2).map_err(|e| e.kind);
        let expected = Err(BFErrorKind::MissingClose);
        assert_eq!(result, expected);
        assert_eq!(result2, expected);
    }

    #[test]
    fn program_from_string() {
        let instructions_str = "+-><[],.";
        let instructions = Instructions::from_string(instructions_str);

        let prgm_from_str = Program::from_string(instructions_str);
        let prgm_from_instructions = Program::new(instructions);

        assert_eq!(prgm_from_str, prgm_from_instructions);
    }

    #[test]
    fn reset_program() {
        let instructions = Instructions::from_string("+-><[],.");
        let mut prgm = Program::new(instructions.clone());
        let static_prgm = Program::new(instructions);

        blank_execute_prgm(&mut prgm).unwrap();

        assert_ne!(prgm, static_prgm);

        prgm.reset();

        assert_eq!(prgm, static_prgm);
    }

    #[test]
    fn instruction_execution() {
        let instructions = Instructions::from_string(
            "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++."
        );
        let mut outstring = String::new();
        let mut program = Program::new(instructions);
        let _ = program.execute(|| ' ', |charout| outstring.push(charout));

        assert_eq!("Hello World!\n", outstring);
    }
}
