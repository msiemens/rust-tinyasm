#![feature(phase, macro_rules, log_syntax, slicing_syntax, trace_macros, unboxed_closures, unboxed_closure_sugar, concat_idents)]

extern crate debug;
extern crate seq;
extern crate serialize;
#[phase(plugin, link)] extern crate log;

extern crate docopt;
#[phase(plugin)]       extern crate seq_macros;
#[phase(plugin)]       extern crate docopt_macros;

#[cfg(not(test))] use std::io::File;
#[cfg(not(test))] use docopt::FlagParser;

use self::instructions::{Memory, Jump, Halt, Void};

mod instructions;


macro_rules! items (
    ( $count:expr, $len:expr ) => (
        box $i as Box<Instruction>
    )
)


docopt!(Args, "
Usage: tiny-vm <source>
")


#[cfg(not(test))]
fn main() {
    let args: Args = FlagParser::parse().unwrap_or_else(|e| e.exit());

    let source = match File::open(&Path::new(args.arg_source)).read_to_end() {
        Ok(v)  => v,
        Err(e) => { println!("Can't read file: {}", e); return; }
    };

    run(source[]);
}

fn run(source: &[u8]) {
    let instructions = instructions::get();

    let mut memory = [0u8, ..256];
    //let mut memory: Vec<u8> = Vec::from_elem(256, 0);
    let mut pc = 0u;

    loop {
        debug!("");
        debug!("source: {}@{}", source, source.len());
        debug!("memory: {}@{}", memory[], memory[].len());
        debug!("pc: {:u}", pc);

        let opcode = source[pc];                        debug!("opcode: {:#04X}", opcode);
        let ref instruction = instructions[opcode];
        let argc = instruction.argc();                  debug!("argc: {}", argc);
        let argv = match argc {
            0 => [][],  // Empty slice
            _ => source[pc + 1 .. pc + 1 + argc]
        };                                              debug!("argv: {}", argv)
        pc += 1 + argc;  // Increment programm counter


        match instruction.execute(argv, memory[]) {
            Halt => break,
            Jump(address) => {
                pc = address as uint;
            },
            Memory(location, content) => {
                debug!("Setting m[{}] = {}", location, content);
                memory[location as uint] = content;
            },
            Void => {
            }
        }
    }
}