use std::collections::HashMap;

use assembler::ast::*;
use assembler::lexer::dummy_source;
use assembler::util::fatal;


pub fn expand(ast: &mut Vec<Statement>) {
    SubroutineExpander {
        ast: ast,
        routines: HashMap::new()
    }.expand()
}


#[deriving(Show, Clone, Eq, PartialEq)]
enum SubroutineState {
    SubroutineStart(Ident),
    InSubroutine,
    SubroutineEnd,
    SubroutineCall(Ident, Vec<MacroArgument>),
    NotInSubroutine
}

struct SubroutineExpander<'a> {
    ast: &'a mut Vec<Statement>,
    routines: HashMap<Ident, uint>
}

impl<'a> SubroutineExpander<'a> {
    fn collect_routines(&mut self) {
        for stmt in self.ast.iter() {
            let (ident, args) = if let StatementMacro(ref ident, ref args) = stmt.node {
                (ident.clone(), args)
            } else {
                continue
            };


            if ident.as_str()[] == "start" {
                if args.len() != 2 {
                    fatal!("Invalid number of arguments for @start: {}",
                           args.len() @ stmt)
                }

                let name = if let MacroArgIdent(ref name) = args[0].node {
                    name.clone()
                } else {
                    fatal!("Expected subroutine name, got {}",
                           args[0] @ stmt)
                };

                let argc = if let MacroArgArgument(ref arg) = args[1].node {
                    if let ArgumentLiteral(argc) = arg.node {
                        argc as uint
                    } else {
                        fatal!("Expected argument count, got {}",
                               args[1] @ stmt)
                    }
                } else {
                    fatal!("Expected argument count, got {}",
                           args[1] @ stmt)
                };

                if self.routines.insert(name, argc).is_some() {
                    fatal!("Redefinition of subroutine: {}", args[0] @ stmt)
                };
            }
        }
    }

    fn build_preamble(&mut self) {
        self.ast.insert(0, Statement::new(
            // $return = [_]
            StatementConst(
                Ident::from_str("return"),
                Argument::new(
                    ArgumentAddress(None),
                    dummy_source()
                )
            ),
            dummy_source()
        ));
        self.ast.insert(1, Statement::new(
            // $jump_back = [_]
            StatementConst(
                Ident::from_str("jump_back"),
                Argument::new(
                    ArgumentAddress(None),
                    dummy_source()
                )
            ),
            dummy_source()
        ));

        for i in range(0, *self.routines.values().max().unwrap()) {
            self.ast.insert(i + 2, Statement::new(
                // $arg{i} = [_]
                StatementConst(
                    Ident::from_string(format!("arg{}", i)),
                    Argument::new(
                        ArgumentAddress(None),
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

            state = match self.ast[i].node {
                StatementMacro(ref ident, ref args) => {
                    match ident.as_str()[] {
                        "start" => {
                            if state == InSubroutine {
                                // Invalid state, shouldn't happen
                                fatal!("Cannot nest subroutines" @ self.ast[i]);
                            }

                            // Get subroutine name
                            let ident = if let MacroArgIdent(ref ident) = args[0].node {
                                ident.clone()
                            } else {
                                fatal!("Expected subroutine name, found `{}`",
                                       args[0].node @ args[0]);
                            };

                            SubroutineStart(ident)
                        },
                        "end" => {
                            if args.len() > 0 {
                                fatal!("@end takes no arguments" @ args[0]);
                            }

                            SubroutineEnd
                        },
                        "call" => {
                            if args.len() == 0 {
                                fatal!("Expected (name, args...), found `)`"
                                       @ self.ast[i]);
                            }

                            // Get subroutine name
                            let ident = if let MacroArgIdent(ref ident) = args[0].node {
                                ident.clone()
                            } else {
                                fatal!("Expected subroutine name, found `{}`",
                                       args[0] @ args[0]);
                            };

                            // Get expected argument count
                            let routine_argc = *self.routines.get(&ident).unwrap_or_else(|| {
                                fatal!("Unknown subroutine: {}", ident
                                       @ self.ast[i]);
                            });

                            if args.len() - 1 != routine_argc {
                                fatal!("Wrong argument count: found {} arguments, expected {}",
                                       args.len() - 1, routine_argc
                                       @ args[0]);
                            }

                            // Get arguments (cloned)
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
                        StatementLabel(ident),
                        loc.clone()
                    );
                    self.ast.insert(i + 1, Statement::new(
                        // MOV $return 0
                        StatementOperation(
                            Mnemonic(from_str("MOV").unwrap()),
                            vec![
                                Argument::new(
                                    ArgumentConst(
                                        Ident::from_str("return")
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
                    // Build subroutine epilogue
                    self.ast[i] = Statement::new(
                        // JMP $jump_back
                        StatementOperation(
                            Mnemonic(from_str("JMP").unwrap()),
                            vec![
                                Argument::new(
                                    ArgumentConst(
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
                    for j in range(0, args.len()) {
                        let arg = match args[j].node {
                            MacroArgArgument(ref arg) => arg,
                            MacroArgIdent(ref ident) => fatal!("Expected argument, got `{}`",
                                                               ident @ args[j])
                        };

                        self.ast.insert(i + j, Statement::new(
                            StatementOperation(
                                // MOV arg{i} {arg_i}
                                Mnemonic(from_str("MOV").unwrap()),
                                vec![
                                    Argument::new(
                                        ArgumentConst(
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
                        StatementOperation(
                            // MOV $jump_back :ret{i}
                            Mnemonic(from_str("MOV").unwrap()),
                            vec![
                                Argument::new(
                                    ArgumentConst(
                                        Ident::from_str("jump_back")
                                    ),
                                    loc.clone()
                                ),
                                Argument::new(
                                    ArgumentLabel(
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
                        StatementOperation(
                            // JMP :{name}
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

                    // Add label where to continue
                    self.ast.insert(i + args.len() + 2, Statement::new(
                        // ret{i}:
                        StatementLabel(Ident::from_string(format!("ret{}", i))),
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

        debug!("Subroutines: {}", self.routines);

        // Pass 2: Replace function definitions
        self.process_macros();

        // Pass 3: Remove macro statements
        self.ast.retain(|stmt| {
            match stmt.node {
                StatementMacro(..) => {
                    false
                },
                _ => true
            }
        });

    }
}