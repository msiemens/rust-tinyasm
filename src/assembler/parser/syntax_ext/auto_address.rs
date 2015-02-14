//! A syntax extension that auto-fills addresses to prevent repeating and
//! having to keeping track of memory addresses.
//!
//! # Example:
//!
//! ```
//! $const = [_]
//! MOV $const 2
//! ```
//!
//! Results in:
//!
//! ```
//! MOV [0] 2
//! ```

use assembler::parser::ast::{Program, Statement, Argument, ArgumentNode, MacroArgument};


pub fn expand(source: &mut Program) {
    // The address to use next
    let mut auto_addr = 0u8;

    // A helper function that replaces the value of the current argument
    // with the next free address.
    let mut update_arg = |arg: &mut ArgumentNode| {
        if let Argument::Address(addr) = arg.value {
            if addr == None {
                arg.value = Argument::Address(Some(auto_addr));
                auto_addr += 1;
            }
        }
    };

    // Process all statements in the current source
    for stmt in source.iter_mut() {
        match stmt.value {

            // Process operation arguments
            Statement::Operation(_, ref mut args) => {
                for arg in args.iter_mut() {
                    update_arg(arg);
                }
            },

            // Process constants
            Statement::Const(_, ref mut arg) => {
                update_arg(arg);
            },

            // Process macro arguments
            Statement::Macro(_, ref mut margs) => {
                for marg in margs.iter_mut() {
                    if let MacroArgument::Argument(ref mut arg) = marg.value {
                        update_arg(arg);
                    }
                }
            }

            _ => {}
        }
    }
}