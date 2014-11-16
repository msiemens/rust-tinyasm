use assembler::instructions::*;
use assembler::ast::*;
use assembler::util::fatal;


pub fn generate_binary(ast: Vec<Statement>) -> Vec<u8> {
    let mut binary = vec![];

    for stmt in ast.iter() {
        match stmt.node {
            StatementOperation(mnem, ref args) => {
                // Get the requested mnemonic
                let Mnemonic(instr) = mnem;

                // Get the argument types we received
                let arg_types = args.iter().map(|ref arg| {
                    match arg.node {
                        ArgumentLiteral(_) | ArgumentChar(_) => Literal,
                        ArgumentAddress(_) => Address,
                        _ => fatal!("Unprocessed argument: {}", arg @ arg)
                    }
                }).collect();

                // Find the opcode matching the given argument types
                let instr_class = INSTRUCTIONS.get(&instr).unwrap();
                let op = instr_class.iter().find(|op| {
                    op.args == arg_types
                }).unwrap_or_else(|| {
                    let allowed_arg_types = instr_class.iter()
                        .map(|ref i| i.args.clone())
                        .map(|args| format!("{}", args))
                        .collect::<Vec<_>>()
                        .connect(" or ");

                    fatal!("Invalid arguments for {}: found {}, allowed: {}",
                           instr, arg_types, allowed_arg_types @ stmt)
                });

                // Finally, write the opcode
                binary.push(op.opcode);

                // Write arguments
                for arg in args.iter() {
                    match arg.node {
                        ArgumentLiteral(i) => binary.push(i),
                        ArgumentChar(c) => binary.push(c),
                        ArgumentAddress(a) => binary.push(a.unwrap()),
                        // Shouldn't happen as we check this in arg_types
                        _ => fatal!("Unprocessed argument: {}", arg @ arg)
                    }
                }

            },
            _ => fatal!("Not an operation: {}", stmt @ stmt)
        }
    }

    binary
}


#[cfg(test)]
mod test {
    use assembler::dummy_source;
    use assembler::ast::*;

    use super::generate_binary;

    #[test]
    fn test_operation() {
        assert_eq!(
            generate_binary(vec![
                Statement::new(
                    StatementOperation(
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