use std::fmt;

use assembler::instructions::Instruction;
use assembler::parser::lexer::SourceLocation;
use assembler::util::SharedString;


pub type Program = Vec<StatementNode>;

macro_rules! define(
    ( $name:ident -> $wrapper:ident : $( $variants:ident ( $( $arg:ty ),* ) ),* ) => {
        #[derive(PartialEq, Eq, Clone)]
        pub struct $wrapper {
            pub value: $name,
            pub location: SourceLocation
        }

        impl_to_string!($wrapper: "{}", value);

        #[derive(PartialEq, Eq, Clone)]
        pub enum $name {
            $( $variants ( $( $arg ),* ) ),*
        }

        impl $name {
            pub fn new(stmt: $name, location: SourceLocation) -> $wrapper {
                $wrapper {
                    value: stmt,
                    location: location
                }
            }
        }
    };
);


define!(Statement -> StatementNode:
    Include(IPath),
    Label(Ident),
    Const(Ident, ArgumentNode),
    Operation(Mnemonic, Vec<ArgumentNode>),
    Macro(Ident, Vec<MacroArgumentNode>)
);

impl fmt::Debug for Statement {
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

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


define!(Argument -> ArgumentNode:
    Literal(u8),
    Address(Option<u8>),
    Const(Ident),
    Label(Ident),
    Char(u8)
);

impl fmt::Debug for Argument {
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

impl fmt::Display for Argument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


define!(MacroArgument -> MacroArgumentNode:
    Argument(ArgumentNode),
    Ident(Ident)
);

impl fmt::Debug for MacroArgument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MacroArgument::Argument(ref arg) => write!(f, "{}", arg),
            MacroArgument::Ident(ref name) => write!(f, "{}", name)
        }
    }
}

impl fmt::Display for MacroArgument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Ident(pub SharedString);

impl Ident {
    pub fn as_str(&self) -> SharedString {
        let Ident(ref s) = *self;
        s.clone()
    }

    pub fn clone(&self) -> Ident {
        Ident(self.as_str())
    }
}

impl fmt::Debug for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


#[derive(PartialEq, Eq, Clone)]
pub struct Mnemonic(pub Instruction);

impl fmt::Debug for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Mnemonic(ref mnem) = *self;
        write!(f, "{:?}", mnem)
    }
}

impl fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


#[derive(PartialEq, Eq, Clone)]
pub struct IPath(pub SharedString);

impl IPath {
    #[cfg(test)]
    pub fn from_str(s: &'static str) -> IPath {
        IPath(rcstr(s))
    }

    /*pub fn from_string(s: String) -> IPath {
        IPath(rcstring(s))
    }*/

    pub fn as_str(&self) -> SharedString {
        let IPath(ref p) = *self;
        p.clone()
    }
}

impl fmt::Debug for IPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let IPath(ref path) = *self;
        write!(f, "<{}>", path)
    }
}

impl fmt::Display for IPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}