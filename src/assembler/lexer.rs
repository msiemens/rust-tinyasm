use super::instructions::Instructions;


#[deriving(PartialEq, Eq, Show)]
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
    IDENT(String),
    INTEGER(u8),
    CHAR(u8),
    PATH(String),

    EOF

    //UNKNOWN(String)
}


pub struct Lexer<'a> {
    source: &'a str,
    pos: uint,
    curr: Option<char>,
    curr_line: uint
}


impl<'a> Lexer<'a> {
    pub fn new(source: &str) -> Lexer {
        Lexer {
            source: source,
            pos: 0,
            curr: Some(source.char_at(0)),
            curr_line: 1
        }
    }

    fn bump(&mut self) {
        self.curr = self.nextch();
        self.pos += 1;

        debug!("Moved on to {}", self.curr)
    }

    fn nextch(&self) -> Option<char> {
        let new_pos = self.pos + 1;
        if new_pos < self.source.len() {
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

            panic!("Expected {}, found {}", expect_str, found_str)
        }

        self.bump();
    }

    fn curr_repr(&self) -> String {
        match self.curr {
            Some(c) => {
                let mut repr = vec![];
                c.escape_default(|r| repr.push(r));
                String::from_chars(repr[])
            },
            None => "EOF".into_string()
        }
    }

    /// Collect a series of chars starting at the current character
    fn collect(&mut self, cond: |&char| -> bool) -> String {
        let mut chars = vec![];

        debug!("start colleting")

        while let Some(c) = self.curr {
            if cond(&c) {
                chars.push(c);
                self.bump();
            } else {
                debug!("colleting finished")
                break
            }
        }

        String::from_chars(chars[])
    }

    fn tokenize_mnemonic(&mut self) -> Token {
        let mnemonic = self.collect(|c| {
            c.is_alphabetic() && c.is_uppercase()
        });

        let mnemonic = match from_str(mnemonic[]) {
            Some(m) => m,
            None => panic!("Invalid mnemonic: {}", mnemonic)
        };

        MNEMONIC(mnemonic)
    }

    fn tokenize_ident(&mut self) -> Token {
        let ident = self.collect(|c| {
            (c.is_alphabetic() && c.is_lowercase()) || *c == '_'
        });

        IDENT(ident)
    }

    fn tokenize_digit(&mut self) -> Token {
        let integer = self.collect(|c| c.is_digit());

        let integer = match from_str(integer[]) {
            Some(m) => m,
            None => panic!("Invalid integer: {}", integer)
        };

        INTEGER(integer)
    }

    fn tokenize_char(&mut self) -> Token {
        self.bump();  // '\'' matched, move on
        let c = self.curr.unwrap_or_else(|| panic!("Expected a char, found EOF"));

        let tok = if c == '\\' {
            // Escaped char, let's take a look on one more char
            //match self.iter.next_char() {
            self.bump();
            match self.curr {
                Some('n') => CHAR(10),
                Some('\'') => CHAR(39),
                Some(c) => panic!("Unsupported or invalid escape sequence: \\{}", c),
                None => panic!("Expected escaped char, found EOF")
            }
        } else if c.is_whitespace() || c.is_alphanumeric() {
            CHAR(c as u8)
        } else {
            panic!("Invalid character: {}", c)
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

        PATH(path)
    }

    /// Read the next token and return it
    fn read_token(&mut self) -> Option<Token> {
        let c = match self.curr {
            Some(c) => c,
            None => { return Some(EOF) }
        };

        let token = match c {
            '#' => { self.bump(); HASH },
            ':' => { self.bump(); COLON },
            '$' => { self.bump(); DOLLAR },
            '@' => { self.bump(); AT },
            ',' => { self.bump(); COMMA },
            '=' => { self.bump(); EQ },
            '_' => { self.bump(); UNDERSCORE },
            '(' => { self.bump(); LPAREN },
            ')' => { self.bump(); RPAREN },
            '[' => { self.bump(); LBRACKET },
            ']' => { self.bump(); RBRACKET },

            c if c.is_alphabetic() && c.is_uppercase() => {
                self.tokenize_mnemonic()
            },
            c if c.is_alphabetic() && c.is_lowercase() => {
                self.tokenize_ident()
            },
            c if c.is_digit() => {
                self.tokenize_digit()
            },
            '\'' => {
                self.tokenize_char()
            },
            '<' => {
                self.tokenize_path()
            },

            ';' => {
                self.collect(|c| *c != '\n');
                return None;
            },
            c if c.is_whitespace() => {
                if c == '\n' {
                    self.curr_line += 1;
                } else if c == '\r' && self.nextch_is('\n') {
                    self.curr_line += 1;
                    self.bump();  // Skip \n
                }

                self.bump();
                return None;
            },
            c => {
                panic!("Unknown token: {}", c)
                // UNKNOWN(format!("{}", c).into_string())
            }
        };

        Some(token)
    }

    pub fn is_eof(&self) -> bool {
        self.curr.is_none()
    }

    pub fn next_token(&mut self) -> Token {
        if self.is_eof() {
            EOF
        } else {
            let mut tok = self.read_token();
            while tok.is_none() {
                // Token is to be ignored, try next one
                tok = self.read_token();
            }

            tok.unwrap()  // Can't really be None any more
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = vec![];

        // NOTE: We can't use `for c in self.iter` because then we can't
        //       access `self.iter` inside the body because it's borrowed.
        while !self.is_eof() {
            debug!("Processing {}", self.curr)
            match self.read_token() {
                Some(t) => tokens.push(t),
                None => {}
            }
            debug!("So far: {}", tokens)
        }

        tokens
    }
}


#[cfg(test)]
mod tests {
    use super::super::instructions::INSTRUCTIONS;
    use super::*;

    #[test]
    fn test_mnemonic() {
        assert_eq!(Lexer::new("MOV").tokenize(),
                   vec![MNEMONIC(from_str("MOV").unwrap())]);
    }

    #[test]
    fn test_ident() {
        assert_eq!(Lexer::new("abc").tokenize(),
                   vec![IDENT("abc".into_string())]);
    }

    #[test]
    fn test_digit() {
        assert_eq!(Lexer::new("128").tokenize(),
                   vec![INTEGER(128)]);
    }

    #[test]
    fn test_char() {
        assert_eq!(Lexer::new("'a'").tokenize(),
                   vec![CHAR('a' as u8)]);
        assert_eq!(Lexer::new("' '").tokenize(),
                   vec![CHAR(' ' as u8)]);
        assert_eq!(Lexer::new("'\n'").tokenize(),
                   vec![CHAR('\n' as u8)]);
        assert_eq!(Lexer::new("'\\\''").tokenize(),
                   vec![CHAR('\'' as u8)]);
    }

    #[test]
    fn test_path() {
        assert_eq!(Lexer::new("<asd>").tokenize(),
                   vec![PATH("asd".into_string())]);
    }

    #[test]
    fn test_comment() {
        assert_eq!(Lexer::new("; asd").tokenize(),
                   vec![]);
        assert_eq!(Lexer::new("; asd\nMOV ;asd\nMOV").tokenize(),
                   vec![MNEMONIC(from_str("MOV").unwrap()),
                        MNEMONIC(from_str("MOV").unwrap())]);
    }

    #[test]
    fn test_whitespace() {
        assert_eq!(Lexer::new("\n\n\n\n     \n\t\n").tokenize(),
                   vec![]);
        assert_eq!(Lexer::new("      MOV        \n\n MOV").tokenize(),
                   vec![MNEMONIC(from_str("MOV").unwrap()),
                        MNEMONIC(from_str("MOV").unwrap())]);
    }

    #[test]
    fn test_line_counter() {
        let mut lx = Lexer::new("MOV\nMOV");
        lx.tokenize();
        assert_eq!(lx.curr_line, 2);

        let mut lx = Lexer::new("MOV\r\nMOV");
        lx.tokenize();
        assert_eq!(lx.curr_line, 2);
    }
}