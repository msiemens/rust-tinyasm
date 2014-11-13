// TODO: Framework for AST processors
// TODO: Implement AST processors

use std::io::{Open, Write};
use std::fmt::{Show, Formatter, FormatError};
#[cfg(not(test))] use std::io::File;
use std::rc::Rc;

#[cfg(not(test))]
use super::Args;


macro_rules! rcstr(
    ($s:expr) => (
        Rc::new($s.into_string())
    )
)


mod instructions;
mod ast;
mod lexer;
mod parser;
mod codegen;
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
    // Read source file
    let source_file = Path::new(args.arg_input);
    let source = match File::open(&source_file).read_to_string() {
        Ok(contents) => contents,
        Err(_) => panic!("Cannot read {}", source_file.display())
    };
    let filename = source_file.str_components().last().unwrap().unwrap();

    let ast = parser::Parser::new(source[], filename).parse();

    if args.flag_v {
        println!("AST:")
        for stmt in ast.iter() {
            println!("{}", stmt);
        }
        println!("")
        println!("Binary:")
    }

    let binary = codegen::generate_binary(ast);

    if args.flag_bin {
        let mut file = File::open_mode(&Path::new(args.arg_output), Open, Write);
        for b in binary.iter() {
            match file.write([*b][]).err() {
                Some(e) => panic!("Cannot write to output file: {}", e),
                None => {}
            }
        }
    } else {
        for b in binary.iter() {
            print!("{:#04X} ", *b)
        }
    }
}