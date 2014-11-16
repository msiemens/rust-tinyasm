use std::fmt;

use assembler::instructions::Instructions;
use assembler::lexer::SourceLocation;
use assembler::util::{SharedString, rcstr, rcstring};


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
    StatementInclude(IPath),
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