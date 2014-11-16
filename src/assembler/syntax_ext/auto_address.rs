use assembler::ast::*;

pub fn expand(ast: &mut Vec<Statement>) {
    let mut auto_addr = 0u8;

    let update_addr = |arg: &mut Argument, addr: Option<u8>| {
        if addr == None {
            arg.node = ArgumentAddress(Some(auto_addr));
            auto_addr += 1;
        }
    };

    let update_arg = |arg: &mut Argument| {
        if let ArgumentAddress(addr) = arg.node {
            update_addr(arg, addr);
        }
    };

    for stmt in ast.iter_mut() {
        match stmt.node {

            StatementOperation(_, ref mut args) => {
                for arg in args.iter_mut() {
                    update_arg(arg);
                }

            },

            StatementConst(_, ref mut arg) => {
                update_arg(arg);
            },

            StatementMacro(_, ref mut margs) => {
                for marg in margs.iter_mut() {
                    if let MacroArgArgument(ref mut arg) = marg.node {
                        update_arg(arg);
                    }
                }
            }

            _ => {}
        }
    }
}