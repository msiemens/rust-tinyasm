use assembler::ast::{AST, Statement, Argument, Argument_, MacroArgument};

pub fn expand(ast: &mut AST) {
    let mut auto_addr = 0u8;

    let update_addr = |arg: &mut Argument_, addr: Option<u8>| {
        if addr == None {
            arg.node = Argument::Address(Some(auto_addr));
            auto_addr += 1;
        }
    };

    let update_arg = |arg: &mut Argument_| {
        if let Argument::Address(addr) = arg.node {
            update_addr(arg, addr);
        }
    };

    for stmt in ast.iter_mut() {
        match stmt.node {

            Statement::Operation(_, ref mut args) => {
                for arg in args.iter_mut() {
                    update_arg(arg);
                }

            },

            Statement::Const(_, ref mut arg) => {
                update_arg(arg);
            },

            Statement::Macro(_, ref mut margs) => {
                for marg in margs.iter_mut() {
                    if let MacroArgument::Argument(ref mut arg) = marg.node {
                        update_arg(arg);
                    }
                }
            }

            _ => {}
        }
    }
}