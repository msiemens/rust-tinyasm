use std::collections::HashMap;

use assembler::ast::{AST, Statement, Argument, Ident};
use assembler::util::{warn, fatal};

pub fn expand(ast: &mut AST) {
    let mut labels: HashMap<Ident, uint> = HashMap::new();
    let mut offset = 0u;

    // Pass 1: Collect label definitions
    ast.retain(|stmt| {
        match stmt.node {
            Statement::Label(ref name) => {
                if labels.insert(name.clone(), offset).is_some() {
                    warn!("Redefinition of label: {}", name @ stmt);
                }

                false
            },

            Statement::Operation(_, ref args) => {
                offset += 1 + args.len();
                true
            },

            _ => {
                true
            }
        }
    });

    debug!("Labels: {}", labels);

    // Pass 2: Replace label usages
    for stmt in ast.iter_mut() {
        if let Statement::Operation(_, ref mut args) = stmt.node {
            for arg in args.iter_mut() {
                // Get a new location if argument is a label
                arg.node = if let Argument::Label(ref name) = arg.node {
                    if let Some(val) = labels.get(name) {
                        overflow_check!(*val @ arg);
                        Argument::Literal(*val as u8)
                    } else {
                        fatal!("Unknown label: {}", name @ arg)
                    }
                } else {
                    continue
                }
            }
        }
    }
}