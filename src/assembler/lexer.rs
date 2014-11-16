use std::fmt;
use std::rc::Rc;

use assembler::instructions::Instructions;
use assembler::util::{fatal, rcstr, SharedString};


#[deriving(PartialEq, Eq, Clone)]
pub struct SourceLocation {
    pub filename: SharedString,
    pub lineno: uint
}

impl fmt::Show for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::FormatError> {
        write!(f, "{}:{}", self.filename, self.lineno)
    }
}

pub fn dummy_source() -> SourceLocation {
    SourceLocation {
        filename: rcstr("<input>"),
        lineno: 0
    }
}


#[deriving(Clone, PartialEq, Eq)]
pub enum Token {
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

    MNEMONIC(Instructions),
    IDENT(SharedString),
    INTEGER(u8),
    CHAR(u8),
    PATH(SharedString),

    EOF,

    PLACEHOLDER
    //UNKNOWN(String)
}

impl fmt::Show for Token {
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

            Token::MNEMONIC(instr)  => write!(f, "{}", instr),
            Token::IDENT(ref ident) => write!(f, "{}", ident),
            Token::INTEGER(i)       => write!(f, "{}", i),
            Token::CHAR(c)          => write!(f, "{:c}", c as char),
            Token::PATH(ref path)   => write!(f, "{}", path),

            Token::EOF         => write!(f, "EOF"),
            Token::PLACEHOLDER => write!(f, "PLACEHOLDER")
        }
    }
}


pub trait Lexer {
    fn get_source(&self) -> SourceLocation;
    fn next_token(&mut self) -> Token;
    fn tokenize(&mut self) -> Vec<Token>;
}


pub struct FileLexer<'a> {
    source: &'a str,
    file: SharedString,
    len: uint,
    pos: uint,
    curr: Option<char>,
    lineno: uint
}

impl<'a> FileLexer<'a> {
    pub fn new(source: &'a str, file: &'a str) -> FileLexer<'a> {
        FileLexer {
            source: source,
            file: rcstr(file),
            len: source.len(),
            pos: 0,
            curr: Some(source.char_at(0)),
            lineno: 1
        }
    }


    fn fatal(&self, msg: String) -> ! {
        fatal(msg, &self.get_source())
    }


    fn is_eof(&self) -> bool {
        self.curr.is_none()
    }

    fn bump(&mut self) {
        self.curr = self.nextch();
        self.pos += 1;

        debug!("Moved on to {}", self.curr)
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

    fn nextch_is(&self, c: char) -> bool {
        self.nextch() == Some(c)
    }

    fn expect(&mut self, expect: char) {
        if self.curr != Some(expect) {
            let expect_str = match expect {
                '\'' => "quote".into_string(),
                c => format!("'{}'", c)
            };
            let found_str = match self.curr {
                Some(_) => format!("'{}'", self.curr_repr()),
                None => "EOF".into_string()
            };

            self.fatal(format!("Expected `{}`, found `{}`",
                               expect_str, found_str))
        }

        self.bump();
    }

    fn curr_repr(&self) -> SharedString {
        match self.curr {
            Some(c) => {
                let mut repr = vec![];
                c.escape_default(|r| repr.push(r));
                Rc::new(String::from_chars(repr[]))
            },
            None => rcstr("EOF")
        }
    }


    /// Collect a series of chars starting at the current character
    fn collect(&mut self, cond: |&char| -> bool) -> SharedString {
        let mut chars = vec![];

        debug!("start colleting")

        while let Some(c) = self.curr {
            if cond(&c) {
                chars.push(c);
                self.bump();
            } else {
                debug!("colleting finished")
                break;
            }
        }

        Rc::new(String::from_chars(chars[]))
    }

    fn eat_all(&mut self, cond: |&char| -> bool) {
        while let Some(c) = self.curr {
            if cond(&c) { self.bump(); }
            else { break; }
        }
    }

    fn tokenize_mnemonic(&mut self) -> Token {
        let mnemonic = self.collect(|c| {
            c.is_alphabetic() && c.is_uppercase()
        });

        let mnemonic = if let Some(m) = from_str(mnemonic[]) {
            m
        } else {
            self.fatal(format!("invalid mnemonic: {}", mnemonic))
        };

        Token::MNEMONIC(mnemonic)
    }

    fn tokenize_ident(&mut self) -> Token {
        let ident = self.collect(|c| {
            (c.is_alphabetic() && c.is_lowercase())
                || c.is_digit()
                || *c == '_'
        });

        Token::IDENT(ident)
    }

    fn tokenize_digit(&mut self) -> Token {
        let integer = self.collect(|c| c.is_digit());

        let integer = if let Some(m) = from_str(integer[]) {
            m
        } else {
            self.fatal(format!("invalid integer: {}", integer))
        };

        Token::INTEGER(integer)
    }

    fn tokenize_char(&mut self) -> Token {
        self.bump();  // '\'' matched, move on
        let c = self.curr.unwrap_or_else(|| {
            self.fatal(format!("expected a char, found EOF"));
        });

        let tok = if c == '\\' {
            // Escaped char, let's take a look on one more char
            //match self.iter.next_char() {
            self.bump();
            match self.curr {
                Some('n')  => Token::CHAR(10),
                Some('\'') => Token::CHAR(39),
                Some(c) => self.fatal(format!("unsupported or invalid escape sequence: \\{}", c)),
                None => self.fatal(format!("expected escaped char, found EOF"))
            }
        } else if c.is_whitespace() || c.is_alphanumeric() {
            Token::CHAR(c as u8)
        } else {
            self.fatal(format!("invalid character: {}", c))
        };
        self.bump();

        // Match closing quote
        self.expect('\'');

        tok
    }

    fn tokenize_path(&mut self) -> Token {
        self.bump();  // '<' matched, move on
        let path = self.collect(|c| *c != '>');

        // Match closing '>'
        self.expect('>');

        Token::PATH(path)
    }

    /// Read the next token and return it
    fn read_token(&mut self) -> Option<Token> {
        let c = match self.curr {
            Some(c) => c,
            None => return Some(EOF)
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
            c if c.is_digit() => self.tokenize_digit(),
            '\''              => self.tokenize_char(),
            '<'               => self.tokenize_path(),

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

impl<'a> Lexer for FileLexer<'a> {
    fn get_source(&self) -> SourceLocation {
        SourceLocation {
            filename: self.file.clone(),
            lineno: self.lineno
        }
    }

    fn next_token(&mut self) -> Token {
        if self.is_eof() {
            Token::EOF
        } else {
            let mut tok = self.read_token();
            while tok.is_none() {
                // Token is to be ignored, try next one
                tok = self.read_token();
            }

            tok.unwrap()  // Can't really be None any more
        }
    }

    #[allow(dead_code)]  // Used for tests
    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = vec![];

        // NOTE: We can't use `for c in self.iter` because then we can't
        //       access `self.iter` inside the body because it's borrowed.
        while !self.is_eof() {
            debug!("Processing {}", self.curr)

            if let Some(t) = self.read_token() {
                tokens.push(t);
            }

            debug!("So far: {}", tokens)
        }

        tokens
    }
}


impl Lexer for Vec<Token> {
    fn get_source(&self) -> SourceLocation {
        dummy_source()
    }

    fn next_token(&mut self) -> Token {
        match self.remove(0) {
            Some(tok) => tok,
            None => Token::EOF
        }
    }

    fn tokenize(&mut self) -> Vec<Token> {
        let mut v = vec![];
        v.push_all(self[]);

        v
    }
}


#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::{Token, Lexer, FileLexer};
    use super::Token::*;
    use assembler::util::rcstr;

    fn tokenize(src: &'static str) -> Vec<Token> {
        FileLexer::new(src, "<test>").tokenize()
    }

    #[test]
    fn test_mnemonic() {
        assert_eq!(tokenize("MOV"),
                   vec![MNEMONIC(from_str("MOV").unwrap())]);
    }

    #[test]
    fn test_ident() {
        assert_eq!(tokenize("abc"),
                   vec![IDENT(rcstr("abc"))]);
    }

    #[test]
    fn test_digit() {
        assert_eq!(tokenize("128"),
                   vec![INTEGER(128)]);
    }

    #[test]
    fn test_char() {
        assert_eq!(tokenize("'a'"),
                   vec![CHAR('a' as u8)]);
        assert_eq!(tokenize("' '"),
                   vec![CHAR(' ' as u8)]);
        assert_eq!(tokenize("'\n'"),
                   vec![CHAR('\n' as u8)]);
        assert_eq!(tokenize("'\\\''"),
                   vec![CHAR('\'' as u8)]);
    }

    #[test]
    fn test_path() {
        assert_eq!(tokenize("<asd>"),
                   vec![PATH(rcstr("asd"))]);
    }

    #[test]
    fn test_comment() {
        assert_eq!(tokenize("; asd"),
                   vec![]);
        assert_eq!(tokenize("; asd\nMOV ;asd\nMOV"),
                   vec![MNEMONIC(from_str("MOV").unwrap()),
                        MNEMONIC(from_str("MOV").unwrap())]);
    }

    #[test]
    fn test_whitespace() {
        assert_eq!(tokenize("\n\n\n\n     \n\t\n"),
                   vec![]);
        assert_eq!(tokenize("      MOV        \n\n MOV"),
                   vec![MNEMONIC(from_str("MOV").unwrap()),
                        MNEMONIC(from_str("MOV").unwrap())]);
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