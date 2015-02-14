#[macro_use] mod util;
mod codegen;
mod instructions;
mod parser;


use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use super::Args;


pub fn main(args: Args) {
    // Read source file
    let input_path = Path::new(&args.arg_input);
    let source = read_file(&input_path);

    // Parse source file
    let filename = input_path.iter().last().unwrap().to_string_lossy();
    let mut source = parser::Parser::new(&source, &filename).parse();

    if args.flag_v {
        println!("Source:");
        for stmt in source.iter() {
            println!("{}", stmt);
        }
        print!("\n");
    }

    // Expand syntax extensions
    parser::expand_syntax_extensions(&mut source);

    if args.flag_v {
        println!("Expanded source:");
        for stmt in source.iter() {
            println!("{}", stmt);
        }
        print!("\n");
    }

    // Generate binary
    let binary = codegen::generate_binary(source);

    if args.flag_bin {
        write_binary(binary, &Path::new(&args.arg_output));
    } else {
        for stmt in binary.iter() {
            for b in stmt.iter() {
                print!("{:#04x} ", *b)
            }
            print!("\n");
        }
    }
}


fn read_file(input_path: &Path) -> String {
    let mut file = match File::open(&input_path) {
        Ok(f) => f,
        Err(err) => panic!("Can't open {}: {}", input_path.display(), err)
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(contents) => contents,
        Err(_) => panic!("Can't read {}", input_path.display())
    };

    contents
}

fn write_binary(binary: Vec<Vec<u8>>, output_path: &Path) {
    let mut file = match File::create(output_path) {
        Ok(f) => f,
        Err(err) => panic!("Can't write to {}: {}", output_path.display(), err)
    };

    for stmt in binary.iter() {
        for b in stmt.iter() {
            match file.write_all(&[*b]) {
                Ok(_) => {},
                Err(err) => panic!("Can't write to {}: {}", output_path.display(), err)
            }
        }
    }
}