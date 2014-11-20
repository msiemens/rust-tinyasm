use assembler::ast::{AST, Statement, Argument, ArgumentNode, MacroArgument};

pub fn expand(ast: &mut AST) {
    let mut auto_addr = 0u8;

    let update_addr = |arg: &mut ArgumentNode, addr: Option<u8>| {
        if addr == None {
            arg.value = Argument::Address(Some(auto_addr));
            auto_addr += 1;
        }
    };

    let update_arg = |arg: &mut ArgumentNode| {
        if let Argument::Address(addr) = arg.value {
            update_addr(arg, addr);
        }
    };

    for stmt in ast.iter_mut() {
        match stmt.value {

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
                    if let MacroArgument::Argument(ref mut arg) = marg.value {
                        update_arg(arg);
                    }
                }
            }

            _ => {}
        }
    }
}