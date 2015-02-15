//! A syntax extension for constants
//!
//! # Example:
//!
//! ```
//! $const = [0]
//! MOV $const 2
//! ```
//!
//! Results in:
//!
//! ```
//! MOV [0] 2
//! ```

use std::collections::HashMap;
use assembler::parser::ast::{Program, Statement, Argument, Ident};


pub fn expand(source: &mut Program) {
    let mut consts: HashMap<Ident, Argument> = HashMap::new();

    // Pass 1: Collect constant definitions & remove them from the source
    source.retain(|stmt| {
        let (name, value) = match stmt.value {
            Statement::Const(ref name, ref value) => (name, value),
            _ => return true  // Not a const assignment, keep it
        };

        // Collect value
        match value.value {
            Argument::Literal(_) | Argument::Address(_) => {
                if consts.insert(name.clone(), value.value.clone()).is_some() {
                    warn!("redefinition of ${:?}", name; value);
                }
            },
            _ => fatal!("invalid constant value: {:?}", value; value)
        }

        false  // Remove the definition from the source
    });

    debug!("Constants: {:?}", consts);

    // Pass 2: Replace usages of constants
    for stmt in source.iter_mut() {
        let args = match stmt.value {
            Statement::Operation(_, ref mut args) => args,
            _ => continue
        };

        for arg in args.iter_mut() {
            // Get the new value if the argument is a constant
            arg.value = if let Argument::Const(ref name) = arg.value {
                match consts.get(name) {
                    Some(value) => value.clone(),
                    None => fatal!("unknown constant: ${:?}", name; arg)
                }
            } else {
                continue
            };
        }
    }
}