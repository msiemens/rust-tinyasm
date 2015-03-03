//! The Lexer
//!
//! Nothing outstanding, just a normal lexer.

use std::borrow::ToOwned;
use std::fmt;
use std::rc::Rc;

use assembler::util::fatal;
use machine::{Mnemonic, WordSize};


// --- Source Location ----------------------------------------------------------

pub type SharedString = Rc<String>;

#[derive(PartialEq, Eq, Clone)]
pub struct SourceLocation {
    pub filename: SharedString,
    pub lineno: usize
}

impl_to_string!(SourceLocation: "{}:{}", filename, lineno);


pub fn dummy_source() -> SourceLocation {
    SourceLocation {
        filename: Rc::new(String::from_str("<input>")),
        lineno: 0
    }
}


// --- List of Tokens -----------------------------------------------------------

#[derive(Clone, PartialEq, Eq)]
pub enum Token<'a> {
    HASH,
    COLON,
    DOLLAR,
    AT,
    COMMA,
    EQ,
    UNDERSCORE,

    LPAREN,
    RPAREN,
    LBRACKET,
    RBRACKET,

    MNEMONIC(Mnemonic),
    IDENT(&'a str),
    INTEGER(WordSize),
    CHAR(WordSize),
    PATH(&'a str),

    EOF,

    PLACEHOLDER
    //UNKNOWN(String)
}

impl<'a> fmt::Debug for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Token::HASH       => write!(f, "#"),
            Token::COLON      => write!(f, ":"),
            Token::DOLLAR     => write!(f, "$"),
            Token::AT         => write!(f, "@"),
            Token::COMMA      => write!(f, ","),
            Token::EQ         => write!(f, "="),
            Token::UNDERSCORE => write!(f, "_"),

            Token::LPAREN     => write!(f, "("),
            Token::RPAREN     => write!(f, ")"),
            Token::LBRACKET   => write!(f, "["),
            Token::RBRACKET   => write!(f, "]"),

            Token::MNEMONIC(ref instr) => write!(f, "{:?}", instr),
            Token::IDENT(ref ident)    => write!(f, "{:?}", ident),
            Token::INTEGER(i)          => write!(f, "{}", i),
            Token::CHAR(c)             => write!(f, "{}", c as char),
            Token::PATH(ref path)      => write!(f, "{:?}", path),

            Token::EOF         => write!(f, "EOF"),
            Token::PLACEHOLDER => write!(f, "PLACEHOLDER")
        }
    }
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


// --- The Lexer ----------------------------------------------------------------
// We use a Lexer trait along with two implementations: FileLexer and Vec<Token>.
// The first one is used for processing a file on the hard drive, the second
// is used for testing purposes.

pub trait Lexer<'a> {
    fn get_source(&self) -> SourceLocation;
    fn next_token(&mut self) -> Token<'a>;
    fn tokenize(&mut self) -> Vec<Token<'a>>;
}


// --- The Lexer: FileLexer -----------------------------------------------------

pub struct FileLexer<'a> {
    source: &'a str,
    file: SharedString,
    len: usize,

    pos: usize,
    curr: Option<char>,

    lineno: usize
}

impl<'a> FileLexer<'a> {

    pub fn new(source: &'a str, file: &str) -> FileLexer<'a> {
        FileLexer {
            source: source,
            file: Rc::new(String::from_str(file)),
            len: source.len(),

            pos: 0,
            curr: Some(source.char_at(0)),

            lineno: 1
        }
    }


    // --- File Lexer: Helpers ---------------------------------------------------

    fn fatal(&self, msg: String) -> ! {
        fatal(msg, &self.get_source())
    }


    fn is_eof(&self) -> bool {
        self.curr.is_none()
    }


    // --- File Lexer: Character processing --------------------------------------

    fn bump(&mut self) {
        self.curr = self.nextch();
        self.pos += 1;

        debug!("Moved on to {:?}", self.curr)
    }

    fn nextch(&self) -> Option<char> {
        let mut new_pos = self.pos + 1;

        // When encountering multi-byte UTF-8, we may stop in the middle
        // of it. Fast forward till we see the next actual char or EOF

        while !self.source.is_char_boundary(new_pos)
                && self.pos < self.len {
            new_pos += 1;
        }

        if new_pos < self.len {
            Some(self.source.char_at(new_pos))
        } else {
            None
        }
    }

    fn curr_repr(&self) -> String {
        match self.curr {
            Some(c) => c.escape_default().collect(),
            None    => "EOF".to_owned()
        }
    }

    fn expect(&mut self, expect: char) {
        if self.curr != Some(expect) {
            // Build error message
            let expect_str = match expect {
                '\'' => String::from_str("quote"),
                c    => format!("'{}'", c)
            };
            let found_str = match self.curr {
                Some(_) => format!("'{}'", self.curr_repr()),
                None    => String::from_str("EOF")
            };

            self.fatal(format!("Expected `{}`, found `{}`",
                               expect_str, found_str))
        }

        self.bump();
    }

    fn collect<F>(&mut self, cond: F) -> &'a str
            where F: Fn(&char) -> bool {
        let start = self.pos;

        debug!("start colleting");

        while let Some(c) = self.curr {
            if cond(&c) {
                self.bump();
            } else {
                debug!("colleting finished");
                break;
            }
        }

        let end = self.pos;

        &self.source[start..end]
    }

    fn eat_all<F>(&mut self, cond: F)
            where F: Fn(&char) -> bool {
        while let Some(c) = self.curr {
            if cond(&c) { self.bump(); }
            else { break; }
        }
    }

    // --- File Lexer: Tokenizers ------------------------------------------------

    fn tokenize_mnemonic(&mut self) -> Token<'a> {
        debug!("Tokenizing a mnemonic");

        let mnemonic_str = self.collect(|c| c.is_alphabetic() && c.is_uppercase());
        let mnemonic     = match mnemonic_str.parse() {
            Ok(m) => m,
            Err(_) => self.fatal(format!("invalid mnemonic: {}", mnemonic_str))
        };

        Token::MNEMONIC(mnemonic)
    }

    fn tokenize_ident(&mut self) -> Token<'a> {
        debug!("Tokenizing an ident");

        let ident = self.collect(|c| {
            (c.is_alphabetic() && c.is_lowercase()) || c.is_numeric() || *c == '_'
        });

        Token::IDENT(ident)
    }

    fn tokenize_digit(&mut self) -> Token<'a> {
        debug!("Tokenizing a digit");

        let integer_str = self.collect(|c| c.is_numeric());
        let integer     = match integer_str.parse() {
            Ok(i) => i,
            Err(_) => self.fatal(format!("invalid integer: {}", integer_str))
        };

        Token::INTEGER(integer)
    }

    fn tokenize_char(&mut self) -> Token<'a> {
        debug!("Tokenizing a char");

        self.bump();  // '\'' matched, move on

        let c = self.curr.unwrap_or_else(|| {
            self.fatal(format!("expected a char, found EOF"));
        });
        let tok = if c == '\\' {
            // Escaped char, let's take a look on one more char
            self.bump();
            match self.curr {
                Some('n')  => Token::CHAR(10),
                Some('\'') => Token::CHAR(39),
                Some(c) => self.fatal(format!("unsupported or invalid escape sequence: \\{}", c)),
                None => self.fatal(format!("expected escaped char, found EOF"))
            }
        } else {
            Token::CHAR(c as WordSize)
        };
        self.bump();

        // Match closing quote
        self.expect('\'');

        tok
    }

    fn tokenize_path(&mut self) -> Token<'a> {
        debug!("Tokenizing a path");

        self.bump();  // '<' matched, move on

        let path = self.collect(|c| *c != '>');

        // Match closing '>'
        self.expect('>');

        Token::PATH(path)
    }

    /// Read the next token and return it
    ///
    /// If `None` is returned, the current token is to be ignored and the
    /// lexer requests the reader to read the next token instead.
    fn read_token(&mut self) -> Option<Token<'a>> {
        let c = match self.curr {
            Some(c) => c,
            None    => return Some(Token::EOF)
        };

        let token = match c {
            '#' => { self.bump(); Token::HASH },
            ':' => { self.bump(); Token::COLON },
            '$' => { self.bump(); Token::DOLLAR },
            '@' => { self.bump(); Token::AT },
            ',' => { self.bump(); Token::COMMA },
            '=' => { self.bump(); Token::EQ },
            '_' => { self.bump(); Token::UNDERSCORE },
            '(' => { self.bump(); Token::LPAREN },
            ')' => { self.bump(); Token::RPAREN },
            '[' => { self.bump(); Token::LBRACKET },
            ']' => { self.bump(); Token::RBRACKET },

            c if c.is_alphabetic() && c.is_uppercase() => {
                self.tokenize_mnemonic()
            },
            c if c.is_alphabetic() && c.is_lowercase() => {
                self.tokenize_ident()
            },
            c if c.is_numeric() => self.tokenize_digit(),
            '\''                => self.tokenize_char(),
            '<'                 => self.tokenize_path(),

            ';' => {
                self.eat_all(|c| *c != '\n');
                return None;
            },
            c if c.is_whitespace() => {
                if c == '\n' { self.lineno += 1; }

                self.bump();
                return None;
            },
            c => {
                self.fatal(format!("unknown token: {}", c))
                // UNKNOWN(format!("{}", c).into_string())
            }
        };

        Some(token)
    }
}

impl<'a> Lexer<'a> for FileLexer<'a> {
    fn get_source(&self) -> SourceLocation {
        SourceLocation {
            filename: self.file.clone(),
            lineno: self.lineno
        }
    }

    fn next_token(&mut self) -> Token<'a> {
        if self.is_eof() {
            Token::EOF
        } else {
            // Read the next token until it's not none
            loop {
                if let Some(token) = self.read_token() {
                    return token;
                }
            }
        }
    }

    #[allow(dead_code)]  // Used for tests
    fn tokenize(&mut self) -> Vec<Token<'a>> {
        let mut tokens = vec![];

        while !self.is_eof() {
            debug!("Processing {:?}", self.curr);

            if let Some(t) = self.read_token() {
                tokens.push(t);
            }

            debug!("So far: {:?}", tokens)
        }

        tokens
    }
}


// --- The Lexer: Vec<Token> ----------------------------------------------------

impl<'a> Lexer<'a> for Vec<Token<'a>> {
    fn get_source(&self) -> SourceLocation {
        dummy_source()
    }

    fn next_token(&mut self) -> Token<'a> {
        if self.len() >= 1 {
            self.remove(0)
        } else {
            Token::EOF
        }
    }

    fn tokenize(&mut self) -> Vec<Token<'a>> {
        self.iter().cloned().collect()
    }
}


// --- Tests --------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::borrow::ToOwned;
    use std::rc::Rc;

    use super::{Token, Lexer, FileLexer};
    use super::Token::*;
    use assembler::util::rcstr;
    use machine::WordSize;

    fn tokenize(src: &'static str) -> Vec<Token> {
        FileLexer::new(src, "<test>").tokenize()
    }

    #[test]
    fn test_mnemonic() {
        assert_eq!(tokenize("MOV"),
                   vec![MNEMONIC("MOV".parse().unwrap())]);
    }

    #[test]
    fn test_ident() {
        assert_eq!(tokenize("abc"),
                   vec![IDENT("abc")]);
    }

    #[test]
    fn test_ident_with_underscore() {
        assert_eq!(tokenize("abc_efg"),
                   vec![IDENT("abc_efg")]);
    }

    #[test]
    fn test_digit() {
        assert_eq!(tokenize("128"),
                   vec![INTEGER(128)]);
    }

    #[test]
    fn test_char() {
        assert_eq!(tokenize("'a'"),
                   vec![CHAR('a' as WordSize)]);
        assert_eq!(tokenize("' '"),
                   vec![CHAR(' ' as WordSize)]);
        assert_eq!(tokenize("'\n'"),
                   vec![CHAR('\n' as WordSize)]);
        assert_eq!(tokenize("'\\\''"),
                   vec![CHAR('\'' as WordSize)]);
    }

    #[test]
    fn test_path() {
        assert_eq!(tokenize("<asd>"),
                   vec![PATH("asd")]);
    }

    #[test]
    fn test_comment() {
        assert_eq!(tokenize("; asd"),
                   vec![]);
        assert_eq!(tokenize("; asd\nMOV ;asd\nMOV"),
                   vec![MNEMONIC("MOV".parse().unwrap()),
                        MNEMONIC("MOV".parse().unwrap())]);
    }

    #[test]
    fn test_whitespace() {
        assert_eq!(tokenize("\n\n\n\n     \n\t\n"),
                   vec![]);
        assert_eq!(tokenize("      MOV        \n\n MOV"),
                   vec![MNEMONIC("MOV".parse().unwrap()),
                        MNEMONIC("MOV".parse().unwrap())]);
    }

    #[test]
    fn test_line_counter() {
        let mut lx = FileLexer::new("MOV\nMOV", "<test>");
        lx.tokenize();
        assert_eq!(lx.lineno, 2);

        let mut lx = FileLexer::new("MOV\r\nMOV", "<test>");
        lx.tokenize();
        assert_eq!(lx.lineno, 2);

        let mut lx = FileLexer::new("#include<lib\\something>", "<test>");
        lx.tokenize();
        assert_eq!(lx.lineno, 1);
    }
}