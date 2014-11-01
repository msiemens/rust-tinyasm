use std::io::{stdout, File, Open, Write};

use super::Args;
use self::instructions::{INSTRUCTIONS, Type, Instructions};


//mod ast;
mod lexer;
mod instructions;


pub fn main(args: Args) {
    let output = if args.flag_bin {
        box File::open_mode(&Path::new(args.arg_output), Open, Write).unwrap() as Box<Writer>
    } else {
        box stdout() as Box<Writer>
    };

    let mut assembler = Assembler {
        binary: args.flag_bin,
        input: Path::new(args.arg_input),
        output: output
    };

    assembler.run()
}


struct Assembler<'a> {
    binary: bool,
    input: Path,
    output: Box<Writer + 'a>  // TODO: Why the lifetime?
}

impl<'a> Assembler<'a> {
    /*
    fn preprocess() {}
    */
    fn write_asm(&mut self, c: u8) {
        let err = if self.binary {
            self.output.write([c])
        } else {
            write!(self.output, "{:#04X} ", c)
        }.err();

        match err {
            Some(e) => panic!("Cannot write to output: {}", e),
            None => {}
        }
    }

    fn assemble_line(&mut self, line: &str) {
        let tokens:Vec<&str> = line.trim().split(' ').collect();

        // Lookup specs in instructions list
        let instruction:Instructions = match from_str(tokens[0]) {
            Some(v) => v,
            None    => panic!("Invalid opcode: '{}'", tokens[0])
        };
        let ref specs = INSTRUCTIONS.find(&instruction).unwrap();

        // Get arguments and look up argument types
        let args = tokens.tail();
        let arg_types:Vec<Type> = args.iter()
            .map(|&a| from_str(a).unwrap())  // Type::from_str doesn't return None
            .collect();

        // Find matching instruction and decode to opcode
        let opcode = specs.values()
            .find(|&spec| { spec.args == arg_types })
            .map(|spec| { spec.opcode })
            .unwrap_or_else(|| panic!("Invalid argument types: {}", arg_types));

        // Convert arguments to integers
        let args:Vec<u8> = args.iter()
            .map(|val| val.trim_chars(['[', ']'][]))
            .map(|val| match from_str(val) {
                Some(v) => v,
                None    => panic!("Not a number or overflow: {}", val)
            })
            .collect();

        debug!("Tokens: {}", tokens);
        debug!("Instruction: {}", instruction);
        debug!("Specs: {}", specs);
        debug!("Args: {}", args);
        debug!("Arg Types: {}", arg_types);
        debug!("Opcode: 0x{:X}", opcode);
        debug!("");

        // Finally assemble opcode + args
        self.write_asm(opcode);

        for arg in args.into_iter() {
            self.write_asm(arg);
        }

        if !self.binary {
            match write!(self.output, "\n") {
                Ok(_) => {},
                Err(e) => panic!("Cannot write to output: {}", e)
            }
        }
    }

    fn assemble(&mut self, source: &str) {
        for line in source.lines() {
            self.assemble_line(line);
        }
    }

    fn run(&mut self) {
        // Read source file
        let source = match File::open(&self.input).read_to_string() {
            Ok(contents) => contents,
            Err(_) => panic!("Cannot read {}", self.input.display())
        };

        println!("Tokens: {}", lexer::Lexer::new(source[]).tokenize())
        self.assemble(source[]);
    }
}