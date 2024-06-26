#[derive(Debug, PartialEq)]
enum Instruct {
    MvPtr(i16),   // move pointer
    MvValue(i16), // signed 16-bit so it can add/subtract about u8
    Output,
    Input,
    OpenLoop,
    CloseLoop,
}

#[derive(Debug, PartialEq)]
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

struct ProgramState {
    instructions: Instructions,
    instruction_ptr: usize,

    cells: Vec<u8>,
    cell_ptr: usize,
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
}
