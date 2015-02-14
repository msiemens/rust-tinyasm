//! A syntax extension for custom subroutines
//!
//! # Example:
//!
//! Subroutine call:
//!
//! ```
//! @call(name, arg1, arg2)
//! ```
//!
//! FIXME: Maybe use @name(arg1, arg2) instead?
//!
//! Subroutine definition:
//!
//! ```
//! @start(name, argc)
//!    ...
//! @end()
//! ```

use std::collections::HashMap;
use assembler::util::fatal;
use assembler::parser::ast::{Program, Statement, StatementNode, Argument, MacroArgument, MacroArgumentNode,
                             Ident};
use assembler::parser::{Parser};

use self::SubroutineState::*;


pub fn expand(source: &mut Program) {
    SubroutineExpander {
        source: source,
        routines: HashMap::new()
    }.expand()
}


// --- Subroutine Expansion: Implementation -------------------------------------

// We use a state machine to keep track of where we are and what is allowed.

#[derive(Debug, Clone, Eq, PartialEq)]
enum SubroutineState {
    SubroutineStart(Ident),  // Definition of a new subroutine
    InSubroutine,            // Subroutine body
    SubroutineEnd,           // End of the body
    SubroutineCall(Ident, Vec<MacroArgumentNode>),  // Call of a subroutine
    NotInSubroutine          // Everything else
}

struct SubroutineExpander<'a> {
    source: &'a mut Program,
    routines: HashMap<Ident, usize>
}

impl<'a> SubroutineExpander<'a> {

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
        self.source.retain(|stmt| {
            match stmt.value {
                Statement::Macro(..) => {
                    false
                },
                _ => true
            }
        });

    }

    /// Collect all subroutine definitions and store them in `self.routines`
    fn collect_routines(&mut self) {
        for stmt in self.source.iter() {
            let (ident, args) = match stmt.value {
                Statement::Macro(ref ident, ref args) => (ident.clone(), args),
                _ => continue
            };

            if *ident.as_str() == "start" {
                // Two args expected: name and number of arguments
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

                // Subroutine definition is valid, store it
                if self.routines.insert(name, argc).is_some() {
                    fatal!("redefinition of subroutine: {}", args[0]; stmt)
                };
            }
        }
    }

    fn parse_and_insert(&mut self, source: &str, pos: usize) {
        let ast = Parser::new(source, "<internal>").parse();

        for (i, stmt) in ast.into_iter().enumerate() {
            self.source.insert(pos + i, stmt)
        }
    }

    /// Build the preamble for the subroutine machinery.
    /// Will only be inserted once at
    ///
    /// Will look like this:
    ///
    /// ```
    /// $return = [_]     ; The return value
    /// $jump_back = [_]  ; The return address
    /// $arg0 = [_]       ; Arguments any subroutine receives
    /// ```
    fn build_preamble(&mut self) {
        let mut template = r###"
            $return = [_]
            $jump_back = [_]
        "###.to_string();

        for i in 0 .. *self.routines.values().max().unwrap() {
            template.push_str(&format!("$arg{} = [_]\n", i));
        }

        self.parse_and_insert(&template, 0);
    }

    /// Process subroutine definitions and calls
    fn process_macros(&mut self) {
        let mut state = NotInSubroutine;

        // We use a indexed iteration here because we'll modify the source as we iterate
        // over it
        let mut i = 0;
        while i < self.source.len() {
            let prev_state = state.clone();

            state = match self.get_state_for(&self.source[i], &state) {
                /// State processing & transitions

                SubroutineStart(ident) => {
                    // Build subroutine preamble
                    self.source.remove(i);

                    let mut template = format!("{}:\n", ident);
                    template.push_str("MOV $return 0\n");

                    self.parse_and_insert(&template, i);

                    InSubroutine
                },

                SubroutineEnd => {
                    // Build subroutine epilogue
                    self.source.remove(i);

                    self.parse_and_insert("JMP $jump_back\n", i);

                    NotInSubroutine
                },

                SubroutineCall(name, args) => {
                    self.source.remove(i);

                    let mut template = String::new();

                    // Build arguments
                    for j in 0 .. args.len() {
                        let arg = match args[j].value {
                            MacroArgument::Argument(ref arg) => arg,
                            MacroArgument::Ident(ref ident) => {
                                fatal!("expected argument, got `{}`", ident; args[j])
                            }
                        };

                        template.push_str(&format!("MOV $arg{} {}\n", j, arg));
                    }

                    // Set jumpback
                    template.push_str(&format!("MOV $jump_back :ret{}\n", i));

                    // Jump to function
                    template.push_str(&format!("JMP :{}\n", name));

                    // Add label where to continue
                    template.push_str(&format!("ret{}:\n", i));

                    self.parse_and_insert(&template, i);

                    prev_state  // Return to previous state
                },

                _ => state // Stay in current state
            };

            i += 1;
        }
    }

    /// Get the current state based on the statement we're currently processing
    fn get_state_for(&self, stmt: &StatementNode, state: &SubroutineState) -> SubroutineState {
        match stmt.value {
            Statement::Macro(ref ident, ref args) => {
                match &**ident.as_str() {
                    "start" => {
                        if *state == InSubroutine { fatal!("can't nest subroutines"; stmt); }

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
                            fatal!("@end takes no args"; args[0]);
                        }

                        SubroutineEnd
                    },
                    "call" => {
                        if args.len() == 0 {
                            fatal!("expected (name, args...), found `)`"; stmt);
                        }

                        // Get subroutine name
                        let ident = if let MacroArgument::Ident(ref ident) = args[0].value {
                            ident.clone()
                        } else {
                            fatal!("expected subroutine name, found `{}`", args[0]; args[0]);
                        };

                        // Verify argument count
                        let routine_argc = *self.routines.get(&ident).unwrap_or_else(|| {
                            fatal!("unknown subroutine: {}", ident; stmt);
                        });

                        if args.len() - 1 != routine_argc {
                            fatal!("wrong argument count: found {} args, expected {}",
                                   args.len() - 1, routine_argc; args[0]);
                        }

                        // Get args (cloned)
                        let args: Vec<_> = args[1..].iter()
                            .cloned()
                            .collect();

                        SubroutineCall(ident, args)
                    }
                    _ => state.clone()
                }
            },
            _ => state.clone()
        }
    }
}