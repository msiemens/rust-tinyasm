use std::fmt;

use assembler::instructions::Instructions;
use assembler::lexer::SourceLocation;
use assembler::util::{SharedString, rcstr, rcstring};


pub type AST = Vec<Statement_>;

// FIXME: Find better wrapper names
macro_rules! define(
    ( $name:ident -> $wrapper:ident : $( $variants:ident ( $( $arg:ty ),* ) ),* ) => {
        #[deriving(PartialEq, Eq, Clone)]
        pub struct $wrapper {
            pub node: $name,
            pub location: SourceLocation
        }

        impl $name {
            pub fn new(stmt: $name, location: SourceLocation) -> $wrapper {
                $wrapper {
                    node: stmt,
                    location: location
                }
            }
        }

        impl fmt::Show for $wrapper {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.node)
            }
        }

        #[deriving(PartialEq, Eq, Clone)]
        pub enum $name {
            $( $variants ( $( $arg ),* ) ),*
        }
    };
)


define!(
Statement -> Statement_:
    Include(IPath),
    Label(Ident),
    Const(Ident, Argument_),
    Operation(Mnemonic, Vec<Argument_>),
    Macro(Ident, Vec<MacroArgument_>)
)

impl fmt::Show for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Statement::Include(ref path) => write!(f, "#include {}", path),
            Statement::Label(ref name)   => write!(f, "{}:", name),
            Statement::Const(ref name, ref value) => {
                write!(f, "${} = {}", name, value)
            },
            Statement::Operation(ref mnem, ref args) => {
                try!(write!(f, "{}", mnem));
                for arg in args.iter() {
                    try!(write!(f, " {}", arg));
                }
                Ok(())
            },
            Statement::Macro(ref name, ref args) => {
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
    Literal(u8),
    Address(Option<u8>),
    Const(Ident),
    Label(Ident),
    Char(u8)
)

impl fmt::Show for Argument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Argument::Literal(i) => write!(f, "{}", i),
            Argument::Address(addr) => {
                match addr {
                    Some(i) => write!(f, "[{}]", i),
                    None => write!(f, "[_]")
                }
            },
            Argument::Const(ref name) => write!(f, "${}", name),
            Argument::Label(ref name) => write!(f, ":{}", name),
            Argument::Char(c) => write!(f, "'{}'", c),
        }
    }
}


define!(
MacroArgument -> MacroArgument_:
    Argument(Argument_),
    Ident(Ident)
)

impl fmt::Show for MacroArgument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MacroArgument::Argument(ref arg) => write!(f, "{}", arg),
            MacroArgument::Ident(ref name) => write!(f, "{}", name)
        }
    }
}


#[deriving(PartialEq, Eq, Hash, Clone)]
pub struct Ident(pub SharedString);

impl Ident {
    pub fn from_str(s: &'static str) -> Ident {
        Ident(rcstr(s))
    }

    pub fn from_string(s: String) -> Ident {
        Ident(rcstring(s))
    }

    pub fn as_str(&self) -> SharedString {
        let Ident(ref s) = *self;
        s.clone()
    }

    pub fn clone(&self) -> Ident {
        Ident(self.as_str())
    }
}

impl fmt::Show for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
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
pub struct IPath(pub SharedString);

impl IPath {
    pub fn from_str(s: &'static str) -> IPath {
        IPath(rcstr(s))
    }

    pub fn from_string(s: String) -> IPath {
        IPath(rcstring(s))
    }

    pub fn as_str(&self) -> SharedString {
        let IPath(ref p) = *self;
        p.clone()
    }
}

impl fmt::Show for IPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let IPath(ref path) = *self;
        write!(f, "<{}>", path)
    }
}