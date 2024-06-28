#[derive(Debug, PartialEq, Clone)]
enum Instruct {
    MvPtr(isize),
    MvValue(isize),
    Output,
    Input,
    OpenLoop,
    CloseLoop,
}

// Process the raw string into instructions first. This allows for optimizations later on in the
// interpretation process
#[derive(Debug, PartialEq, Clone)]
pub struct Instructions(Vec<Instruct>);

impl Instructions {
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

// NOTE: The Program struct takes ownership of the instructions.
#[derive(Debug, PartialEq)]
pub struct Program {
    instructions: Instructions,
    instruction_ptr: usize,

    cells: Vec<u8>,
    cell_ptr: usize,

    loop_stack: Vec<usize>,
}

impl Program {
    pub fn new(instructions: Instructions) -> Program {
        Program {
            instructions,
            instruction_ptr: 0,
            cells: vec![],
            cell_ptr: 0,
            loop_stack: vec![],
        }
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

        assert_eq!("Hello World!", outstring);
    }
}
