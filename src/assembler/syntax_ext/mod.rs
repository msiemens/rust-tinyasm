use std::collections::HashMap;
use std::rc::Rc;

use super::{SharedString, dummy_source};
use assembler::ast::*;
use assembler::parser::Parser;
use assembler::util::{fatal, warn};


pub fn expand_syntax_extensions(ast: &mut Vec<Statement>) {
    expand_imports(ast);
    expand_subroutines(ast);
    expand_auto_address(ast);
    expand_constants(ast);
    expand_labels(ast);
}

fn expand_imports(ast: &mut Vec<Statement>) {
    // TODO
}

#[deriving(Show, Clone)]
enum SubroutineState<'a> {
    SubroutineStart(Ident),
    InSubroutine,
    SubroutineEnd,
    SubroutineCall(Ident, Vec<MacroArgument>),
    NotInSubroutine
}

fn expand_subroutines(ast: &mut Vec<Statement>) {
    let mut routines: HashMap<Ident, uint> = HashMap::new();

    // Pass 1: Collect definitions and build preamble
    for stmt in ast.iter() {
        match stmt.node {
            StatementMacro(ref ident, ref args) => {
                let Ident(ref name) = *ident;
                match name[] {
                    "start" => {
                        if args.len() != 2 {
                            fatal(format!("Invalid number of arguments for @start: {}",
                                          args.len()),
                                  &stmt.location)
                        }
                        let sr_name = match args[0].node {
                            MacroArgIdent(ref ident) => ident.clone(),
                            _ => fatal(format!("Expected subroutine name, got {}",
                                               args[0]),
                                       &stmt.location)
                        };
                        let arg_count = match args[1].node {
                            MacroArgArgument(ref arg) => {
                                match arg.node {
                                    ArgumentLiteral(i) => i as uint,
                                    _ => fatal(format!("Expected argument count, got {}",
                                                       args[1]),
                                               &stmt.location)
                                }
                            },
                            _ => fatal(format!("Expected argument count, got {}",
                                               args[1]),
                                       &stmt.location)
                        };
                        match routines.insert(sr_name, arg_count) {
                            Some(v) => {}, // TODO
                            None => {}
                        }
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }

    if routines.len() == 0 {
        return;
    }

    // Build preamble
    ast.insert(0, Statement::new(
        StatementConst(
            Ident(rcstr!("return")),
            Argument::new(
                ArgumentAddress(None),
                dummy_source()
            )
        ),
        dummy_source()
    ));
    ast.insert(1, Statement::new(
        StatementConst(
            Ident(rcstr!("jump_back")),
            Argument::new(
                ArgumentAddress(None),
                dummy_source()
            )
        ),
        dummy_source()
    ));

    for i in range(0, *routines.values().max().unwrap()) {
        ast.insert(i + 2, Statement::new(
            StatementConst(
                Ident(Rc::new(format!("arg{}", i))),
                Argument::new(
                    ArgumentAddress(None),
                    dummy_source()
                )
            ),
            dummy_source()
        ));
    }

    println!("Subroutines: {}", routines);

    let mut i = 0;
    let mut state = NotInSubroutine;
    let mut callcounter = 0u;

    // Pass 2: Replace function definitions
    while i < ast.len() {
        let loc = ast[i].location.clone();
        let prev_state = state.clone();

        state = match ast[i].node {
            StatementMacro(ref ident, ref args) => {
                let Ident(ref name) = *ident;
                //println!("before: {}", state);

                match name[] {
                    "start" => {
                        match state {
                            InSubroutine => panic!("Missing @end()"),
                            _ => {}
                        }

                        match args[0].node {
                            MacroArgIdent(ref ident) => SubroutineStart(ident.clone()),
                            _ => panic!("invalid argument: {}", args[0])
                        }
                    },
                    "end" => {
                        if args.len() > 0 {
                            panic!("@end takes no arguments")
                        }

                        SubroutineEnd
                    },
                    "call" => {
                        let sr_name = match args[0].node {
                            MacroArgIdent(ref ident) => ident.clone(),
                            _ => panic!("invalid argument: {}", args[0])
                        };

                        //println!("{}", sr_name);

                        let argc = *routines.find(&sr_name)
                                           .unwrap_or_else(|| panic!("No such routine: {}", sr_name));

                        if args.len() - 1 != argc {
                            panic!("Wrong argument count: got {}, expected {}", args.len() - 1, argc);
                        }

                        //println!("{}", args[1..]);
                        let args = args[1..].iter().map(|a| a.clone()).collect::<Vec<_>>();

                        SubroutineCall(sr_name, args)
                    }
                    _ => state
                }
            },
            _ => state
        };

        //println!("in process: {}", state);

        state = match state {
            SubroutineStart(ident) => {
                ast[i] = Statement::new(
                    StatementLabel(ident),
                    loc.clone()
                );
                ast.insert(i + 1, Statement::new(
                    StatementOperation(
                        Mnemonic(from_str("MOV").unwrap()),
                        vec![
                            Argument::new(
                                ArgumentConst(
                                    Ident(rcstr!("return"))
                                ),
                                loc.clone()
                            ),
                            Argument::new(
                                ArgumentLiteral(0),
                                loc.clone()
                            )
                        ]
                    ),
                    loc.clone()
                ));

                InSubroutine
            },
            SubroutineEnd => {
                ast[i] = Statement::new(
                    StatementOperation(
                        Mnemonic(from_str("JMP").unwrap()),
                        vec![
                            Argument::new(
                                ArgumentConst(
                                    Ident(rcstr!("jump_back"))
                                ),
                                loc.clone()
                            )
                        ]
                    ),
                    loc.clone()
                );

                NotInSubroutine
            },
            SubroutineCall(name, args) => {
                ast.remove(i);

                for j in range(0, args.len()) {
                    let arg = match args[j].node {
                        MacroArgArgument(ref arg) => arg,
                        MacroArgIdent(ref ident) => panic!("Expected argument, got {}", ident)
                    };

                    ast.insert(i + j, Statement::new(
                        StatementOperation(
                            Mnemonic(from_str("MOV").unwrap()),
                            vec![
                                Argument::new(
                                    ArgumentConst(
                                        Ident(Rc::new(format!("arg{}", j)))
                                    ),
                                    loc.clone()
                                ),
                                arg.clone()
                            ]
                        ),
                        loc.clone()
                    ));
                }

                ast.insert(i + args.len(), Statement::new(
                    StatementOperation(
                        Mnemonic(from_str("MOV").unwrap()),
                        vec![
                            Argument::new(
                                ArgumentConst(
                                    Ident(rcstr!("jump_back"))
                                ),
                                loc.clone()
                            ),
                            Argument::new(
                                ArgumentLabel(
                                    Ident(Rc::new(format!("ret{}", callcounter)))
                                ),
                                loc.clone()
                            )
                        ]
                    ),
                    loc.clone()
                ));

                ast.insert(i + args.len() + 1, Statement::new(
                    StatementOperation(
                        Mnemonic(from_str("JMP").unwrap()),
                        vec![
                            Argument::new(
                                ArgumentLabel(
                                    name
                                ),
                                loc.clone()
                            )
                        ]
                    ),
                    loc.clone()
                ));

                ast.insert(i + args.len() + 2, Statement::new(
                    StatementLabel(Ident(Rc::new(format!("ret{}", callcounter)))),
                    loc.clone()
                ));

                callcounter += 1;

                prev_state
            },
            _ => state
        };

        //println!("then: {}", state);

        i += 1;
    }

    // Pass 2: Replace constant usages
    ast.retain(|stmt| {
        match stmt.node {
            StatementMacro(ref name, ref args) => {
                let Ident(ref name) = *name;
                if name[] == "call" {

                }

                false
            },
            _ => true
        }
    });

}

fn expand_constants(ast: &mut Vec<Statement>) {
    let mut consts: HashMap<Ident, Argument_> = HashMap::new();

    // Pass 1: Collect constant definitions
    ast.retain(|stmt| {
        match stmt.node {
            StatementConst(ref name, ref value) => {
                match value.node {
                    ArgumentLiteral(_) | ArgumentAddress(_) => {
                        match consts.insert(name.clone(), value.node.clone()) {
                            Some(_) => warn(format!("Redefinition of ${}", name),
                                            &value.location),
                            None => {}
                        }
                    },
                    _ => fatal(format!("Invalid argument: {}", value),
                               &value.location)
                }

                false
            },
            _ => true
        }
    });

    println!("Constants: {}", consts);

    // Pass 2: Replace constant usages
    for stmt in ast.iter_mut() {
        match stmt.node {
            StatementOperation(_, ref mut args) => {
                for arg in args.iter_mut() {
                    // Get a new value if the argument is a constant
                    let new_value = match arg.node {
                        ArgumentConst(ref name) => {
                            match consts.get(name) {
                                Some(val) => {
                                    Some(val.clone())
                                },
                                None => fatal(format!("Unknown constant: ${}", name),
                                              &arg.location)
                            }
                        },
                        _ => None
                    };

                    if new_value.is_some() {
                        arg.node = new_value.unwrap();
                    }
                }
            },
            _ => {}
        }
    }
}

fn expand_labels(ast: &mut Vec<Statement>) {
    let mut labels: HashMap<Ident, uint> = HashMap::new();
    let mut offset = 0u;

    // Pass 1: Collect label definitions
    ast.retain(|stmt| {
        match stmt.node {
            StatementLabel(ref name) => {
                match labels.insert(name.clone(), offset) {
                    Some(_) => warn(format!("Redefinition of label: {}", name),
                                    &stmt.location),
                    None => {}
                }
                false
            },
            StatementOperation(_, ref args) => {
                offset += 1 + args.len();
                true
            },
            _ => {
                true
            }
        }
    });

    println!("Labels: {}", labels);

    // Pass 2: Replace label usages
    for stmt in ast.iter_mut() {
        match stmt.node {
            StatementOperation(_, ref mut args) => {
                for arg in args.iter_mut() {
                    // Get a new location if the argument is a label
                    let loc = match arg.node {
                        ArgumentLabel(ref name) => {
                            match labels.get(name) {
                                Some(val) => {
                                    if *val > 255 {
                                        warn(format!("Jump to address > 255: {}", val),
                                             &arg.location)
                                    }
                                    Some(ArgumentLiteral(*val as u8))
                                },
                                None => fatal(format!("Unknown label: {}", name),
                                              &arg.location)
                            }
                        },
                        _ => None
                    };

                    if loc.is_some() {
                        arg.node = loc.unwrap();
                    }
                }
            },
            _ => {}
        }
    }
}

fn expand_auto_address(ast: &mut Vec<Statement>) {
    let mut auto_addr = 0u8;

    let update_addr = |arg: &mut Argument, addr: Option<u8>| {
        if addr.is_none() {
            arg.node = ArgumentAddress(Some(auto_addr));
            auto_addr += 1;
        }
    };

    let update_arg = |arg: &mut Argument| {
        match arg.node {
            ArgumentAddress(addr) => {
                update_addr(arg, addr);
            },
            _ => {}
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
                    match marg.node {
                        MacroArgArgument(ref mut arg) => {
                            update_arg(arg);
                        },
                        _ => {}
                    }
                }
            }

            _ => {}
        }
    }
}