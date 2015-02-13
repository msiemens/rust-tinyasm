#![feature(plugin, slicing_syntax)]

#![feature(rustc_private)]
#![feature(test)]
#![feature(collections)]
#![feature(io)]
#![feature(fs)]
#![feature(hash)]
#![feature(core)]
#![feature(path)]

#![plugin(docopt_macros)]

extern crate ansi_term;
extern crate rand;
extern crate term;
extern crate test;
#[macro_use] extern crate log;

extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;
#[macro_use] extern crate seq;
#[macro_use] extern crate lazy_static;


use docopt::Docopt;


mod assembler;
mod vm;


docopt!(Args derive Debug, "
Usage: tiny asm [-v] <input>
       tiny asm [-v] --bin <input> <output>
       tiny vm <input>
       tiny --help

Options:
    --help  Show this screen.
");


#[cfg(not(test))]
fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

    if args.cmd_asm {
        assembler::main(args)
    } else {
        vm::main(args)
    }
}