use std::old_io::{Open, Write};
#[cfg(not(test))] use std::old_io::File;

#[cfg(not(test))] use super::Args;


#[macro_use] mod util;
mod instructions;
mod ast;
mod lexer;
mod parser;
mod syntax_ext;
mod codegen;


#[cfg(not(test))]
pub fn main(args: Args) {
    // Read source file
    let source_file = Path::new(args.arg_input);
    let source = match File::open(&source_file).read_to_string() {
        Ok(contents) => contents,
        Err(_) => panic!("Cannot read {}", source_file.display())
    };
    let filename = source_file.str_components().last().unwrap().unwrap();

    let mut ast = parser::Parser::new(&*source, filename).parse();

    if args.flag_v {
        println!("AST:");
        for stmt in ast.iter() {
            println!("{}", stmt);
        }
        println!("");
    }

    syntax_ext::expand_syntax_extensions(&mut ast);

    if args.flag_v {
        println!("Expanded AST:");
        for stmt in ast.iter() {
            println!("{}", stmt);
        }
        println!("");
    }

    let binary = codegen::generate_binary(ast);

    if args.flag_bin {
        let mut file = File::open_mode(&Path::new(args.arg_output), Open, Write);
        for stmt in binary.iter() {
            for b in stmt.iter() {
                match file.write(&[*b]).err() {
                    Some(e) => panic!("Cannot write to output file: {}", e),
                    None => {}
                }
            }
        }
    } else {
        for stmt in binary.iter() {
            for b in stmt.iter() {
                print!("{:#04x} ", *b)
            }
            println!("");
        }
    }
}