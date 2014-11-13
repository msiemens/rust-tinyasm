use std::fmt;

use assembler::instructions::Instructions;
use super::{SharedString, SourceLocation};


macro_rules! define(
    ( $name:ident -> $inner:ident : $( $variants:ident ( $( $arg:ty ),* ) ),* ) => {
        #[deriving(PartialEq, Eq, Clone)]
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

        #[deriving(PartialEq, Eq, Clone)]
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

impl fmt::Show for Statement_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StatementInclude(ref path) => write!(f, "#include {}", path),
            StatementLabel(ref name)  => write!(f, "{}:", name),
            StatementConst(ref name, ref value) => {
                write!(f, "${} = {}", name, value)
            },
            StatementOperation(ref mnem, ref args) => {
                try!(write!(f, "{}", mnem));
                for arg in args.iter() {
                    try!(write!(f, " {}", arg));
                }
                Ok(())
            },
            StatementMacro(ref name, ref args) => {
                write!(f, "@{}({})", name,
                       args.iter()
                           .map(|arg| format!("{}", arg))
                           .collect::<Vec<_>>()
                           .connect(" "))
            }
        }
    }
}


define!(
Argument -> Argument_:
    ArgumentLiteral(u8),
    ArgumentAddress(Option<u8>),
    ArgumentConst(Ident),
    ArgumentLabel(Ident),
    ArgumentChar(u8)
)

impl fmt::Show for Argument_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ArgumentLiteral(i) => write!(f, "{}", i),
            ArgumentAddress(addr) => {
                match addr {
                    Some(i) => write!(f, "[{}]", i),
                    None => write!(f, "[_]")
                }
            },
            ArgumentConst(ref name) => write!(f, "${}", name),
            ArgumentLabel(ref name) => write!(f, ":{}", name),
            ArgumentChar(c) => write!(f, "'{}'", c),
        }
    }
}


define!(
MacroArgument -> MacroArgument_:
    MacroArgArgument(Argument),
    MacroArgIdent(Ident)
)

impl fmt::Show for MacroArgument_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MacroArgArgument(ref arg) => write!(f, "{}", arg),
            MacroArgIdent(ref name) => write!(f, "{}", name)
        }
    }
}


#[deriving(PartialEq, Eq, Hash, Clone)]
pub struct Ident(pub SharedString);

impl Ident {
    pub fn clone(&self) -> Ident {
        let Ident(ref s) = *self;
        Ident(s.clone())
    }
}

impl fmt::Show for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Ident(ref ident) = *self;
        write!(f, "{}", ident)
    }
}


#[deriving(PartialEq, Eq, Clone)]
pub struct Mnemonic(pub Instructions);

impl fmt::Show for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Mnemonic(mnem) = *self;
        write!(f, "{}", mnem)
    }
}


#[deriving(PartialEq, Eq, Clone)]
pub struct Path(pub SharedString);

impl fmt::Show for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Path(ref path) = *self;
        write!(f, "<{}>", path)
    }
}