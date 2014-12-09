#![feature(phase, macro_rules, slicing_syntax, globs)]

extern crate term;
extern crate test;
extern crate seq;
extern crate serialize;
#[phase(plugin, link)] extern crate log;

extern crate docopt;
#[phase(plugin)] extern crate seq_macros;
#[phase(plugin)] extern crate docopt_macros;
#[phase(plugin)] extern crate lazy_static;


use docopt::Docopt;


mod assembler;
mod vm;


docopt!(Args deriving Show, "
Usage: tiny asm [-v] <input>
       tiny asm [-v] --bin <input> <output>
       tiny vm <input>
       tiny --help

Options:
    --help  Show this screen.
")


#[cfg(not(test))]
fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

    if args.cmd_asm {
        assembler::main(args)
    } else {
        vm::main(args)
    }
}