#![feature(plugin, slicing_syntax)]

extern crate term;
extern crate test;
#[plugin] #[macro_use] extern crate log;

extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;
#[plugin] #[no_link]   extern crate docopt_macros;
#[plugin] #[macro_use] extern crate seq;
#[plugin] #[macro_use] extern crate lazy_static;


use docopt::Docopt;


mod assembler;
mod vm;


docopt!(Args derive Show, "
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