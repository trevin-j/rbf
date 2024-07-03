#![warn(missing_docs)]

//! # RBF -- Rust BrainF***
//!
//! RBF is a BrainF*** interpreter and soon to be compiler.
//!
//! **TODO: Update this documentation once main.rs is done.**
//!
//! RBF can be installed via `cargo install rbf`. Alternatively, you may use the crate as a library
//! by adding it as a dependency e.g. `cargo add rbf`.
//!
//! **TODO: Add main.rs usage**
//!
//! Usage as a library is simple.
//!
//! # Example
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
//!     // Create some closures to tell the interpreter what to do for io operations.
//!     // On a `,` take input. Here we define the input as reading a single char from the terminal.
//!     let charin = || { term.read_char().expect("Error getting input.") };
//!     // On a `.` we output a char. We define a closure to print and flush the output char.
//!     let charout = |c| {
//!         print!("{}", c);
//!         stdout.flush().expect("Error flushing output.");
//!     };
//!
//!     // Execute the instructions and print if we get an error.
//!     match prgm.execute(charin, charout) {
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
#[derive(Debug, PartialEq)]
pub struct Program {
    instructions: Instructions,
    instruction_ptr: usize,

    cells: Vec<u8>,
    cell_ptr: usize,

    loop_stack: Vec<usize>,
}

impl Program {
    /// Create a new program struct.
    ///
    /// Note that the instructions are moved into the Program struct's ownership.
    pub fn new(instructions: Instructions) -> Program {
        Program {
            instructions,
            instruction_ptr: 0,
            cells: vec![],
            cell_ptr: 0,
            loop_stack: vec![],
        }
    }

    pub fn from_string(instructions: &str) -> Program {
        Self::new(Instructions::from_string(instructions))
    }

    pub fn execute<Fin, Fout>(&mut self, mut input: Fin, mut output: Fout) -> Result<(), String>
    where
        Fin: FnMut() -> char,
        Fout: FnMut(char),
    {
        loop {
            if self.done() {
                break;
            }
            self.step(|| input(), |c| output(c))?;
        }

        Ok(())
    }

    pub fn step<Fin, Fout>(&mut self, input: Fin, output: Fout) -> Result<(), String>
    where
        Fin: FnOnce() -> char,
        Fout: FnOnce(char),
    {
        // Make sure cells length is good so any possible operations we do work.
        self.validate_cells_length();

        let instruction = match self.instructions.0.get(self.instruction_ptr) {
            Some(i) => i,
            None => return Err(format!("Error: The current instruction pointer points to non existing instruction. Instruction pointer: {}. This may have been caused by continuing to call Program::step after it has already finished.", self.instruction_ptr)),
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

    pub fn done(&self) -> bool {
        self.instruction_ptr >= self.instructions.0.len()
    }

    fn move_cell_pointer(&mut self, amount: &isize) -> Result<(), String> {
        self.cell_ptr = match self.cell_ptr.checked_add_signed(*amount) {
            Some(val) => val,
            None => {
                return Err(String::from(
                    "Attempted to access cell out of bounds, likely before index 0.",
                ))
            }
        };

        Ok(())
    }

    // Check the cells length and make sure it's long enough
    // that cell_ptr is a valid index.
    fn validate_cells_length(&mut self) {
        while self.cells.len() <= self.cell_ptr {
            self.cells.push(0);
        }
    }

    fn move_cell_value(&mut self, amount: &isize) {
        self.cells[self.cell_ptr] = self.cells[self.cell_ptr].wrapping_add_signed(*amount as i8);
    }

    fn input_cell<F>(&mut self, input: F) -> Result<(), String>
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
            Err(String::from(
                "Input received a character larger than one byte.",
            ))
        }
    }

    fn output_cell<F>(&self, output: F)
    where
        F: FnOnce(char),
    {
        output(self.cells[self.cell_ptr] as char);
    }

    fn open_loop(&mut self) -> Result<(), String> {
        if self.cells[self.cell_ptr] > 0 {
            self.loop_stack.push(self.instruction_ptr);
        } else {
            match self.move_to_closed_loop() {
                Ok(()) => (),
                Err(e) => return Err(e),
            };
        }

        Ok(())
    }

    fn close_loop(&mut self) -> Result<(), String> {
        self.instruction_ptr = match self.loop_stack.pop() {
            Some(n) => n,
            None => {
                return Err(format!(
                    "There is a close bracket with no matching opening bracket."
                ))
            }
        } - 1;
        Ok(())
    }

    fn move_to_closed_loop(&mut self) -> Result<(), String> {
        let mut loopstack: Vec<usize> = vec![];
        let mut current_instruction = self.instruction_ptr + 1; // We don't want to add
                                                                // current open loop to stack
        loop {
            let instruction = match self.instructions.0.get(current_instruction) {
                Some(i) => i,
                None => {
                    return Err(format!(
                        "There is an open bracket with no matching closing bracket."
                    ))
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

#[cfg(test)]
mod tests {
    use super::*;

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
        let instructions = Instructions::from_string("+_<>[],.");
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
