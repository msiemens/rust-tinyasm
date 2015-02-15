#![feature(plugin, slicing_syntax)]

// Use of unstable libraries
#![feature(collections)]
#![feature(core)]
#![feature(fs)]
#![feature(io)]
#![feature(hash)]
#![feature(path)]
#![feature(os)]
#![feature(std_misc)]

#![plugin(docopt_macros)]
//#![plugin(rest_easy)]

extern crate ansi_term;
extern crate docopt;
extern crate env_logger;
extern crate rand;
extern crate "rustc-serialize" as rustc_serialize;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
#[macro_use] extern crate seq;

use docopt::Docopt;

mod assembler;
mod machine;
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
    env_logger::init().unwrap();

    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

    if args.cmd_asm {
        assembler::main(args)
    } else {
        vm::main(args)
    }
}