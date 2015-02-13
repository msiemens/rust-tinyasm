use std::collections::HashMap;

use assembler::ast::{AST, Statement, Argument, MacroArgument, MacroArgumentNode,
                     Ident, Mnemonic};
use assembler::lexer::dummy_source;
use assembler::util::fatal;

use self::SubroutineState::*;


pub fn expand(ast: &mut AST) {
    SubroutineExpander {
        ast: ast,
        routines: HashMap::new()
    }.expand()
}


#[derive(Debug, Clone, Eq, PartialEq)]
enum SubroutineState {
    SubroutineStart(Ident),
    InSubroutine,
    SubroutineEnd,
    SubroutineCall(Ident, Vec<MacroArgumentNode>),
    NotInSubroutine
}

struct SubroutineExpander<'a> {
    ast: &'a mut AST,
    routines: HashMap<Ident, usize>
}

impl<'a> SubroutineExpander<'a> {
    fn collect_routines(&mut self) {
        for stmt in self.ast.iter() {
            let (ident, args) = if let Statement::Macro(ref ident, ref args) = stmt.value {
                (ident.clone(), args)
            } else {
                continue
            };


            if *ident.as_str() == "start" {
                if args.len() != 2 {
                    fatal!("invalid number of Argument::s for @start: {}",
                           args.len(); stmt)
                }

                let name = if let MacroArgument::Ident(ref name) = args[0].value {
                    name.clone()
                } else {
                    fatal!("expected subroutine name, got {}", args[0]; stmt)
                };

                let argc = if let MacroArgument::Argument(ref arg) = args[1].value {
                    if let Argument::Literal(argc) = arg.value {
                        argc as usize
                    } else {
                        fatal!("expected argument count, got {}", args[1]; stmt)
                    }
                } else {
                    fatal!("expected argument count, got {}", args[1]; stmt)
                };

                if self.routines.insert(name, argc).is_some() {
                    fatal!("redefinition of subroutine: {}", args[0]; stmt)
                };
            }
        }
    }

    fn build_preamble(&mut self) {
        self.ast.insert(0, Statement::new(
            // $return = [_]
            Statement::Const(
                Ident::from_str("return"),
                Argument::new(
                    Argument::Address(None),
                    dummy_source()
                )
            ),
            dummy_source()
        ));
        self.ast.insert(1, Statement::new(
            // $jump_back = [_]
            Statement::Const(
                Ident::from_str("jump_back"),
                Argument::new(
                    Argument::Address(None),
                    dummy_source()
                )
            ),
            dummy_source()
        ));

        for i in 0 .. *self.routines.values().max().unwrap() {
            self.ast.insert(i + 2, Statement::new(
                // $arg{i} = [_]
                Statement::Const(
                    Ident::from_string(format!("arg{}", i)),
                    Argument::new(
                        Argument::Address(None),
                        dummy_source()
                    )
                ),
                dummy_source()
            ));
        }
    }

    fn process_macros(&mut self) {
        let mut i = 0;
        let mut state = NotInSubroutine;

        while i < self.ast.len() {
            let loc = self.ast[i].location.clone();
            let prev_state = state.clone();

            // TODO: If let-ify

            state = match self.ast[i].value {
                Statement::Macro(ref ident, ref args) => {
                    match &**ident.as_str() {
                        "start" => {
                            if state == InSubroutine {
                                // Invalid state, shouldn't happen
                                fatal!("subroutines"; self.ast[i]);
                            }

                            // Get subroutine name
                            let ident = if let MacroArgument::Ident(ref ident) = args[0].value {
                                ident.clone()
                            } else {
                                fatal!("expected subroutine name, found `{}`", args[0].value; args[0]);
                            };

                            SubroutineStart(ident)
                        },
                        "end" => {
                            if args.len() > 0 {
                                fatal!("@end takes no s"; args[0]);
                            }

                            SubroutineEnd
                        },
                        "call" => {
                            if args.len() == 0 {
                                fatal!("expected (name, args...), found `)`"; self.ast[i]);
                            }

                            // Get subroutine name
                            let ident = if let MacroArgument::Ident(ref ident) = args[0].value {
                                ident.clone()
                            } else {
                                fatal!("expected subroutine name, found `{}`", args[0]; args[0]);
                            };

                            // Get expected argument count
                            let routine_argc = *self.routines.get(&ident).unwrap_or_else(|| {
                                fatal!("unknown subroutine: {}", ident; self.ast[i]);
                            });

                            if args.len() - 1 != routine_argc {
                                fatal!("wrong argument count: found {} Argument::s, expected {}",
                                       args.len() - 1, routine_argc; args[0]);
                            }

                            // Get Argument::s (cloned)
                            let args: Vec<_> = args[1..].iter()
                                .map(|a| a.clone())
                                .collect();

                            SubroutineCall(ident, args)
                        }
                        _ => state
                    }
                },
                _ => state
            };

            state = match state {
                SubroutineStart(ident) => {
                    // Build subroutine preamble
                    self.ast[i] = Statement::new(
                        // ident:
                        Statement::Label(ident),
                        loc.clone()
                    );
                    self.ast.insert(i + 1, Statement::new(
                        // MOV $return 0
                        Statement::Operation(
                            Mnemonic("MOV".parse().unwrap()),
                            vec![
                                Argument::new(
                                    Argument::Const(
                                        Ident::from_str("return")
                                    ),
                                    loc.clone()
                                ),
                                Argument::new(
                                    Argument::Literal(0),
                                    loc.clone()
                                )
                            ]
                        ),
                        loc.clone()
                    ));

                    InSubroutine
                },

                SubroutineEnd => {
                    // Build subroutine epilogue
                    self.ast[i] = Statement::new(
                        // JMP $jump_back
                        Statement::Operation(
                            Mnemonic("JMP".parse().unwrap()),
                            vec![
                                Argument::new(
                                    Argument::Const(
                                        Ident::from_str("jump_back")
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
                    self.ast.remove(i);

                    // Build arguments
                    for j in 0 .. args.len() {
                        let arg = match args[j].value {
                            MacroArgument::Argument(ref arg) => arg,
                            MacroArgument::Ident(ref ident) => {
                                fatal!("expected argument, got `{}`", ident; args[j])
                            }
                        };

                        self.ast.insert(i + j, Statement::new(
                            Statement::Operation(
                                // MOV arg{i} {arg_i}
                                Mnemonic("MOV".parse().unwrap()),
                                vec![
                                    Argument::new(
                                        Argument::Const(
                                            Ident::from_string(format!("arg{}", j))
                                        ),
                                        loc.clone()
                                    ),
                                    arg.clone()
                                ]
                            ),
                            loc.clone()
                        ));
                    }

                    // Set jumpback
                    self.ast.insert(i + args.len(), Statement::new(
                        Statement::Operation(
                            // MOV $jump_back :ret{i}
                            Mnemonic("MOV".parse().unwrap()),
                            vec![
                                Argument::new(
                                    Argument::Const(
                                        Ident::from_str("jump_back")
                                    ),
                                    loc.clone()
                                ),
                                Argument::new(
                                    Argument::Label(
                                        Ident::from_string(format!("ret{}", i))
                                    ),
                                    loc.clone()
                                )
                            ]
                        ),
                        loc.clone()
                    ));

                    // Jump to function
                    self.ast.insert(i + args.len() + 1, Statement::new(
                        Statement::Operation(
                            // JMP :{name}
                            Mnemonic("JMP".parse().unwrap()),
                            vec![
                                Argument::new(
                                    Argument::Label(
                                        name
                                    ),
                                    loc.clone()
                                )
                            ]
                        ),
                        loc.clone()
                    ));

                    // Add label where to continue
                    self.ast.insert(i + args.len() + 2, Statement::new(
                        // ret{i}:
                        Statement::Label(Ident::from_string(format!("ret{}", i))),
                        loc.clone()
                    ));

                    prev_state  // Return to previous state
                },

                _ => state // Stay in current state
            };

            i += 1;
        }
    }

    fn expand(&mut self) {
        // Pass 1: Collect definitions and build preamble
        self.collect_routines();
        if self.routines.len() == 0 {
            return
        }

        // Build preamble
        self.build_preamble();

        debug!("Subroutines: {:?}", self.routines);

        // Pass 2: Replace function definitions
        self.process_macros();

        // Pass 3: Remove macro statements
        self.ast.retain(|stmt| {
            match stmt.value {
                Statement::Macro(..) => {
                    false
                },
                _ => true
            }
        });

    }
}