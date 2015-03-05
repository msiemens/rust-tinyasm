use std::fmt;
use ast::{BinOp, UnOp, Ident};
use driver::get_interner;

// --- List of tokens -----------------------------------------------------------

#[derive(Copy, Eq, PartialEq, Hash)]
pub enum Token {
    BinOp(BinOp),
    UnOp(UnOp),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Semicolon,
    RArrow,
    Eq,

    Keyword(Keyword),
    Ident(Ident),
    Type(Ident),
    Int(u32),
    Char(char),

    EOF,
    PLACEHOLDER
}

impl Token {
    pub fn ty(&self) -> TokenType {
        match *self {
            Token::BinOp(op) => match op {
                BinOp::Add      => TokenType::Add,
                BinOp::Sub      => TokenType::Sub,
                BinOp::Mul      => TokenType::Mul,
                BinOp::Div      => TokenType::Div,
                BinOp::Mod      => TokenType::Mod,
                BinOp::Pow      => TokenType::Pow,
                BinOp::And      => TokenType::And,
                BinOp::Or       => TokenType::Or,
                BinOp::BitXor   => TokenType::BitXor,
                BinOp::BitAnd   => TokenType::BitAnd,
                BinOp::BitOr    => TokenType::BitOr,
                BinOp::Shl      => TokenType::Shl,
                BinOp::Shr      => TokenType::Shr,
                BinOp::EqEq     => TokenType::EqEq,
                BinOp::Lt       => TokenType::Lt,
                BinOp::Le       => TokenType::Le,
                BinOp::Ne       => TokenType::Ne,
                BinOp::Ge       => TokenType::Ge,
                BinOp::Gt       => TokenType::Gt,
            },
            Token::UnOp(op) => match op {
                UnOp::Not => TokenType::Not,
                UnOp::Neg => TokenType::Neg,
            },
            Token::LParen       => TokenType::LParen,
            Token::RParen       => TokenType::RParen,
            Token::LBrace       => TokenType::LBrace,
            Token::RBrace       => TokenType::RBrace,
            Token::Comma        => TokenType::Comma,
            Token::Colon        => TokenType::Colon,
            Token::Semicolon    => TokenType::Semicolon,
            Token::RArrow       => TokenType::RArrow,
            Token::Eq           => TokenType::Eq,

            Token::Keyword(kw)  => {
                match kw {
                    Keyword::True   => TokenType::True,
                    Keyword::False  => TokenType::False,
                    _               => TokenType::Keyword
                }
            },
            Token::Ident(..)    => TokenType::Ident,
            Token::Type(..)     => TokenType::Type,
            Token::Int(..)      => TokenType::Int,
            Token::Char(..)     => TokenType::Char,

            _ => panic!("Invalid token type")
        }
    }
}

#[derive(Copy, Eq, PartialEq, Hash, Debug)]
pub enum TokenType {
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Semicolon,
    RArrow,
    Eq,

    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    And,
    Or,
    BitXor,
    BitAnd,
    BitOr,
    Shl,
    Shr,
    EqEq,
    Lt,
    Le,
    Ne,
    Ge,
    Gt,

    Not,
    Neg,

    Keyword,
    True,
    False,
    Ident,
    Type,
    Bool,
    Int,
    Char,
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Token::*;

        match *self {
            BinOp(ref op)       => write!(f, "{:?}", op),
            UnOp(ref op)        => write!(f, "{:?}", op),

            LParen              => write!(f, "("),
            RParen              => write!(f, ")"),
            LBrace              => write!(f, "{{"),
            RBrace              => write!(f, "}}"),
            Comma               => write!(f, ","),
            Colon               => write!(f, ":"),
            Semicolon           => write!(f, ";"),
            RArrow              => write!(f, "->"),
            Eq                  => write!(f, "="),

            Int(i)              => write!(f, "{}", i),
            Char(c)             => write!(f, "{}", c),

            Keyword(ref kw)     => write!(f, "{:?}", kw),
            Ident(id)           => write!(f, "{:?}", id),
            Type(ty)            => write!(f, "{:?}", ty),
            Token::EOF          => write!(f, "EOF"),
            Token::PLACEHOLDER  => write!(f, "PLACEHOLDER")
        }
    }
}


// --- List of keywords ---------------------------------------------------------

macro_rules! keywords(
    ($($kw:ident => $name:expr),*) => {
        #[derive(Copy, Eq, PartialEq, Hash)]
        pub enum Keyword {
            $($kw),*
        }

        impl fmt::Debug for Keyword {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                use self::Keyword::*;

                match *self {
                    $(
                        $kw => write!(f, $name)
                    ),*
                }
            }
        }

        pub fn intern_keywords() {
            let interner = get_interner();
            $( interner.intern($name); )*
        }

        pub fn lookup_keyword(s: &str) -> Option<Keyword> {
            use self::Keyword::*;

            match s {
                $(
                    $name => Some($kw),
                )*
                _ => None
            }
        }
    };
);

keywords! {
    Break   => "break",
    Const   => "const",
    Else    => "else",
    False   => "false",
    Fn      => "fn",
    If      => "if",
    Impl    => "impl",
    Let     => "let",
    Return  => "return",
    Static  => "static",
    True    => "true",
    While   => "while"
}