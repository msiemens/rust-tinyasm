use std::collections::HashMap;

use assembler::ast::{AST, Statement, Argument, Ident};
use assembler::util::{warn, fatal};

pub fn expand(ast: &mut AST) {
    let mut consts: HashMap<Ident, Argument> = HashMap::new();

    // Pass 1: Collect constant definitions
    ast.retain(|stmt| {
        let (name, value) = if let Statement::Const(ref name, ref value) = stmt.node {
            (name, value)
        } else {
            // Not a const assignment, keep it
            return false
        };

        match value.node {
            Argument::Literal(_) | Argument::Address(_) => {
                if consts.insert(name.clone(), value.node.clone()).is_some() {
                    warn!("Redefinition of ${}", name @ value);
                }
            },
            _ => fatal!("Invalid constant value: {}", value @ value)
        }

        false  // Remove this statement from the AST
    });

    debug!("Constants: {}", consts);

    // Pass 2: Replace constant usages
    for stmt in ast.iter_mut() {
        let args = if let Statement::Operation(_, ref mut args) = stmt.node {
            args
        } else {
            continue
        };

        for arg in args.iter_mut() {
            // Get the new value if the argument is a constant
            arg.node = if let Argument::Const(ref name) = arg.node {
                match consts.get(name) {
                    Some(value) => value.clone(),
                    None => fatal!("Unknown constant: ${}", name @ arg)
                }
            } else {
                continue
            };
        }
    }
}