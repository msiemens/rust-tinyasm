use assembler::instructions::{INSTRUCTIONS, ArgumentType};
use assembler::ast::{Statement, Statement_, Argument, Mnemonic};
use assembler::util::fatal;


pub fn generate_binary(ast: Vec<Statement_>) -> Vec<Vec<u8>> {
    let mut binary = vec![];

    for stmt in ast.iter() {
        if let Statement::Operation(mnem, ref args) = stmt.node {
            // Get the requested mnemonic
            let Mnemonic(instr) = mnem;

            // Get the argument types we received
            let arg_types = args.iter().map(|ref arg| {
                match arg.node {
                    Argument::Literal(_) | Argument::Char(_) => {
                        ArgumentType::Literal
                    },
                    Argument::Address(_) => {
                        ArgumentType::Address
                    },
                    _ => fatal!("Unprocessed argument: {}", arg @ arg)
                }
            }).collect();

            // Find the opcode matching the given argument types
            let instr_class = INSTRUCTIONS.get(&instr).unwrap();
            let op = instr_class.iter().find(|op| {
                op.args == arg_types
            }).unwrap_or_else(|| {
                // Build allowed arguments string
                let allowed_arg_types = instr_class.iter()
                    .map(|ref i| i.args.clone())
                    .map(|args| format!("{}", args))
                    .collect::<Vec<_>>()
                    .connect(" or ");

                fatal!("Invalid arguments for {}: found {}, allowed: {}",
                       instr, arg_types, allowed_arg_types @ stmt)
            });

            // Finally, write the opcode
            let mut binary_stmt = vec![op.opcode];
            binary_stmt.extend(args.iter().map(|arg| {
                match arg.node {
                    Argument::Literal(i) => i,
                    Argument::Char(c) => c,
                    Argument::Address(a) => a.unwrap(),
                    // Shouldn't happen as we check this in arg_types
                    _ => fatal!("Unprocessed argument: {}", arg @ arg)
                }
            }));

            binary.push(binary_stmt);
        } else {
            fatal!("Unprocessed operation: {}", stmt @ stmt)
        }
    }

    binary
}


#[cfg(test)]
mod test {
    use assembler::ast::{Statement, Mnemonic};
    use assembler::lexer::dummy_source;

    use super::generate_binary;

    #[test]
    fn test_operation() {
        assert_eq!(
            generate_binary(vec![
                Statement::new(
                    Statement::Operation(
                        Mnemonic(from_str("HALT").unwrap()),
                        vec![]
                    ),
                    dummy_source()
                )
            ]),
            vec![0xFF]
        )
    }
}