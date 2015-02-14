use assembler::instructions::{INSTRUCTIONS, ArgumentType};
use assembler::parser::ast::{Statement, StatementNode, Argument, Mnemonic};
use assembler::util::fatal;


pub fn generate_binary(ast: Vec<StatementNode>) -> Vec<Vec<u8>> {
    let mut binary = vec![];

    for stmt in ast.iter() {
        if let Statement::Operation(ref mnem, ref args) = stmt.value {
            // Get the requested mnemonic
            let Mnemonic(instr) = mnem.clone();

            // Get the argument types we received
            let arg_types: Vec<ArgumentType> = args.iter().map(|ref arg| {
                match arg.value {
                    Argument::Literal(_) | Argument::Char(_) => {
                        ArgumentType::Literal
                    },
                    Argument::Address(_) => {
                        ArgumentType::Address
                    },
                    _ => fatal!("unprocessed argument: {}", arg; arg)
                }
            }).collect();

            // Find the opcode matching the given argument types
            let instr_class = INSTRUCTIONS.get(&instr).unwrap();
            let op = instr_class.iter().find(|op| {
                op.args == arg_types
            }).unwrap_or_else(|| {
                // Build allowed arguments string
                let allowed_arg_types = instr_class.iter()
                    .cloned()
                    .map(|args| format!("{:?}", args))
                    .collect::<Vec<_>>()
                    .connect(" or ");

                fatal!("invalid arguments for {:?}: found {:?}, allowed: {:?}",
                       instr, arg_types, allowed_arg_types; stmt)
            });

            // Finally, write the opcode
            let mut binary_stmt = vec![op.opcode];
            binary_stmt.extend(args.iter().map(|arg| {
                match arg.value {
                    Argument::Literal(i) => i,
                    Argument::Char(c) => c,
                    Argument::Address(a) => a.unwrap(),
                    // Shouldn't happen as we check this in arg_types
                    _ => fatal!("unprocessed argument: {}", arg; arg)
                }
            }));

            binary.push(binary_stmt);
        } else {
            fatal!("unprocessed operation: {}", stmt; stmt)
        }
    }

    binary
}


#[cfg(test)]
mod test {
    use assembler::parser::ast::{Statement, Mnemonic};
    use assembler::parser::dummy_source;

    use super::generate_binary;

    #[test]
    fn test_operation() {
        assert_eq!(
            generate_binary(vec![
                Statement::new(
                    Statement::Operation(
                        Mnemonic("HALT".parse().unwrap()),
                        vec![]
                    ),
                    dummy_source()
                )
            ]),
            vec![vec![0xFF]]
        )
    }
}