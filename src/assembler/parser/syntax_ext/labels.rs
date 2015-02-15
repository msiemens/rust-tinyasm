//! A syntax extension that replaces labels with the referenced instruction number
//!
//! # Example:
//!
//! ```
//! label:
//! GOTO :label
//! ```
//!
//! Results in:
//!
//! ```
//! GOTO 0
//! ```

use std::collections::HashMap;
use assembler::parser::ast::{Program, Statement, Argument, Ident};


pub fn expand(source: &mut Program) {
    let mut labels: HashMap<Ident, usize> = HashMap::new();
    let mut offset = 0us;

    // Pass 1: Collect label definitions
    source.retain(|stmt| {
        match stmt.value {
            // Store label name and current offset
            Statement::Label(ref name) => {
                if labels.insert(name.clone(), offset).is_some() {
                    warn!("redefinition of label: {:?}", name; stmt);
                }

                false  // Remove label definition from the source
            },

            // Increment the offset (only operation statements will count
            // in the final binary)
            Statement::Operation(_, ref args) => {
                offset += 1 + args.len();
                true  // Not a label definition, keep it
            },

            _ => true  // Something else, keep it
        }
    });

    debug!("Labels: {:?}", labels);

    // Pass 2: Replace label usages
    for stmt in source.iter_mut() {

        // Process all operations
        if let Statement::Operation(_, ref mut args) = stmt.value {
            for arg in args.iter_mut() {

                // Get a new location if argument is a label
                arg.value = if let Argument::Label(ref name) = arg.value {

                    if let Some(val) = labels.get(name) {
                        Argument::Literal(overflow_check!(*val, arg))
                    } else {
                        fatal!("unknown label: {:?}", name; arg)
                    }

                } else {
                    continue
                }

            }
        }
    }
}