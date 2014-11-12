use assembler::instructions::*;
use assembler::ast::*;


pub fn generate_binary(ast: Vec<Statement>) -> Vec<u8> {
    let mut binary = vec![];

    for stmt in ast.iter() {
        match stmt.node {
            StatementOperation(mnem, ref args) => {
                let Mnemonic(instr) = mnem;
                let arg_types = args.iter().map(|ref arg| {
                    match arg.node {
                        ArgumentLiteral(_) => Literal,
                        ArgumentAddress(_) => Address,
                        _ => panic!("Invalid argument: {}", arg)
                    }
                }).collect();

                let op = INSTRUCTIONS.get(&instr).unwrap().values().filter(|op| {
                    op.args == arg_types
                }).next().unwrap().opcode;

                binary.push(op);

                for arg in args.iter() {
                    match arg.node {
                        ArgumentLiteral(i) => binary.push(i),
                        ArgumentAddress(a) => match a {
                            Some(a) => binary.push(a),
                            None => panic!("Automem not implemented yet")
                        },
                        _ => panic!("Invalid argument: {}", arg)
                    }
                }

            },
            _ => panic!("Not an operation: {}", stmt)
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