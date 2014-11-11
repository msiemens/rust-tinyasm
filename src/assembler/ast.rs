use std::fmt;

use assembler::instructions::Instructions;
use super::{SharedString, SourceLocation};


macro_rules! define(
    ( $name:ident -> $inner:ident : $( $variants:ident ( $( $arg:ty ),* ) ),* ) => {
        pub struct $name {
            pub node: $inner,
            pub location: SourceLocation
        }

        impl $name {
            pub fn new(stmt: $inner, location: SourceLocation) -> $name {
                $name {
                    node: stmt,
                    location: location
                }
            }
        }

        impl fmt::Show for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.node)
            }
        }

        #[deriving(Show)]
        pub enum $inner {
            $( $variants ( $( $arg ),* ) ),*
        }
    };
)


define!(
Statement -> Statement_:
    StatementInclude(Path),
    StatementLabel(Ident),
    StatementConst(Ident, Argument),
    StatementOperation(Mnemonic, Vec<Argument>),
    StatementMacro(Ident, Vec<MacroArgument>)
)

define!(
Argument -> Argument_:
    ArgumentLiteral(u8),
    ArgumentAddress(Option<u8>),
    ArgumentConst(Ident),
    ArgumentLabel(Ident),
    ArgumentChar(u8)
)

define!(
MacroArgument -> MacroArgument_:
    MacroArgArgument(Argument),
    MacroArgIdent(Ident)
)


#[deriving(Show)]
pub struct Ident(pub SharedString);

#[deriving(Show)]
pub struct Mnemonic(pub Instructions);

#[deriving(Show)]
pub struct Path(pub SharedString);