// TODO: Code cleanup (str!, ...)
// TODO: Implement code generation
// TODO: Framework for AST processors
// TODO: Implement AST processors

use std::fmt::{Show, Formatter, FormatError};
#[cfg(not(test))] use std::io::File;
use std::rc::Rc;

#[cfg(not(test))]
use super::Args;


mod instructions;
mod ast;
mod lexer;
mod parser;
mod util;


type SharedString = Rc<String>;

#[deriving(PartialEq, Eq)]
pub struct SourceLocation {
    pub filename: String,
    pub lineno: uint
}

impl Show for SourceLocation {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        write!(f, "{}:{}", self.filename, self.lineno)
    }
}

pub fn dummy_source() -> SourceLocation {
    SourceLocation {
        filename: "<input>".into_string(),
        lineno: 0
    }
}



#[cfg(not(test))]
pub fn main(args: Args) {
    let mut assembler = Assembler {
        input: Path::new(args.arg_input),
    };

    assembler.run()
}


#[cfg(not(test))]
struct Assembler<'a> {
    input: Path,
}

#[cfg(not(test))]
impl<'a> Assembler<'a> {
    fn run(&mut self) {
        // Read source file
        let source = match File::open(&self.input).read_to_string() {
            Ok(contents) => contents,
            Err(_) => panic!("Cannot read {}", self.input.display())
        };
        // FIXME: More beautiful code!
        let filename = self.input.str_components().last().unwrap().unwrap();

        println!("AST:")
        for stmt in parser::Parser::new(source[], filename).parse().iter() {
            println!("{}", stmt);
        }
    }
}