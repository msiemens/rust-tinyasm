#![feature(phase,macro_rules,log_syntax,slicing_syntax)]

extern crate test;
extern crate seq;
extern crate serialize;
#[phase(plugin, link)] extern crate log;

extern crate docopt;
#[phase(plugin)]       extern crate seq_macros;
#[phase(plugin)]       extern crate docopt_macros;

use std::mem;
#[cfg(not(test))] use std::io::File;
#[cfg(not(test))] use docopt::FlagParser;

use self::instructions::{InstructionSet, Type, Instructions};

mod instructions;

static mut binary: bool = false;


docopt!(Args, "
Usage: tiny-asm [--bin] <source>

Options:
    -b, --bin  Assemble as binary.
")


#[cfg(not(test))]
fn main() {
    let args: Args = FlagParser::parse().unwrap_or_else(|e| e.exit());

    let source = match File::open(&Path::new(args.arg_source)).read_to_string() {
        Ok(v)  => v,
        Err(e) => { println!("Can't read file: {}", e); return; }
    };

    unsafe {
        binary = args.flag_bin;
    }

    print!("{}", assemble(source[]));
}

/* Implement later
fn preprocess() {

}*/

fn format_asm(c: u8) -> String {
    let c = c;
    unsafe {
        if binary {
            let s: &str = mem::transmute([c][]);
            s.into_string()
        } else {
            format!("{:#04X} ", c)
        }
    }
}

fn assemble_line(line: &str, instructions: &InstructionSet) -> String {
    let tokens:Vec<&str> = line.trim().split(' ').collect();

    // Lookup specs in instructions list
    let instruction:Instructions = match from_str(tokens[0]) {
        Some(v) => v,
        None    => fail!("Invalid opcode: '{}'", tokens[0])
    };
    let ref specs = instructions[instruction];

    // Get arguments and look up argument types
    let args = tokens.tail();
    let arg_types:Vec<Type> = args.iter()
        .map(|&a| from_str(a).unwrap())  // Type::from_str doesn't return None
        .collect();

    // Find matching instruction and decode to opcode
    let opcode = specs.values()
        .find(|&spec| { spec.args == arg_types })
        .map(|spec| { spec.opcode })
        .unwrap_or_else(|| fail!("Invalid argument types: {}", arg_types));

    // Convert arguments to integers
    let args:Vec<u8> = args.iter()
        .map(|val| val.trim_chars(['[', ']'][]))
        .map(|val| match from_str(val) {
            Some(v) => v,
            None    => fail!("Not a number or overflow: {}", val)
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
    let mut bin = format_asm(opcode);
    bin.reserve_additional(args.len());

    for arg in args.into_iter() {
        bin.push_str(format_asm(arg)[])
    }

    unsafe {
        if !binary {
            return bin[].trim().into_string();
        }
    }

    bin
}

fn assemble(source: &str) -> String {
    let instructions = instructions::get();

    let hexcode:Vec<String> = source.lines()
        .map(|line| assemble_line(line, &instructions))
        .collect();

    unsafe {
        if binary {
            hexcode.connect("")
        } else {
            hexcode.connect("\n")
        }
    }
}

#[cfg(test)]
mod tests {
    use test::Bencher;
    use super::assemble;

    #[bench]
    fn bench_assemble(b: &mut Bencher) {
        b.iter(|| {
            assemble(r#"Mov [2] 0
Mov [3] 0
Jeq 6 [3] [1]
Add [3] 1
Add [2] [0]
Jmp 2
Mov [0] [2]
Halt"#)
        })
    }

    #[test]
    fn test_assemble() {
        let source = r#"Mov [2] 0
Mov [3] 0
Jeq 6 [3] [1]
Add [3] 1
Add [2] [0]
Jmp 2
Mov [0] [2]
Halt"#;
        let expected = r#"0x08 0x02 0x00
0x08 0x03 0x00
0x15 0x06 0x03 0x01
0x0B 0x03 0x01
0x0A 0x02 0x00
0x0F 0x02
0x07 0x00 0x02
0xFF"#;

        let assembly = assemble(source);
        assert_eq!(assembly[], expected);
    }
}
