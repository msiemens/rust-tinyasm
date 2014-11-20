use std::collections::DList;

use assembler::ast::{
    AST, Statement, Argument, MacroArgument, Mnemonic, Ident, IPath,
    StatementNode, ArgumentNode, MacroArgumentNode
};
use assembler::lexer::{SourceLocation, Lexer, FileLexer, Token};
use assembler::util::{fatal, rcstr};


pub struct Parser<'a> {
    location: SourceLocation,
    token: Token,
    buffer: DList<Token>,
    lexer: Box<Lexer + 'a>
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, file: &'a str) -> Parser<'a> {
        Parser::with_lexer(box FileLexer::new(source, file))
    }

    pub fn with_lexer(lx: Box<Lexer + 'a>) -> Parser<'a> {
        let mut lx = lx;

        Parser {
            token: lx.next_token(),
            location: lx.get_source(),
            buffer: DList::new(),
            lexer: lx
        }
    }


    fn fatal(&self, msg: String) -> ! {
        fatal(msg, &self.location);
    }

    fn unexpected_token(&self, tok: &Token, expected: Option<&'static str>) -> ! {
        match expected {
            Some(ex) => self.fatal(format!("unexpected token: `{}`, expected {}", tok, ex)),
            None => self.fatal(format!("unexpected token: `{}`", tok))
        }
    }

    fn bump(&mut self) {
        self.token = match self.buffer.pop_front() {
            Some(tok) => tok,
            None => self.lexer.next_token()
        };
    }

    fn update_location(&mut self) -> SourceLocation {
        self.location = self.lexer.get_source();
        self.location.clone()
    }

    fn eat(&mut self, tok: &Token) -> bool {
        if self.token == *tok {
            self.bump();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, tok: &Token) {
        if !self.eat(tok) {
            self.fatal(format!("expected `{}`, found `{}`", tok, self.token))
        }
    }

    pub fn look_ahead<R>(&mut self, distance: uint, f: |&Token| -> R) -> R {
        if self.buffer.len() < distance {
            for _ in range(0, distance - self.buffer.len()) {
                self.buffer.push_back(self.lexer.next_token());
            }
        }

        f(self.buffer.iter().nth(distance - 1).unwrap())
    }


    pub fn parse(&mut self) -> AST {
        let mut ast = vec![];

        debug!("Starting parsing")

        while self.token != Token::EOF {
            ast.push(self.parse_statement());
        }

        debug!("Parsing finished")

        ast
    }

    // -------------------------------------------------------------------

    fn token_is_argument(&mut self) -> bool {
        match self.token {
            Token::INTEGER(_) | Token::CHAR(_)
                | Token::LBRACKET | Token::COLON => true,
            Token::DOLLAR => self.look_ahead(2, |t| return t != &Token::EQ),
            _ => false
        }
    }

    // -------------------------------------------------------------------

    fn parse_ident(&mut self) -> Ident {
        let ident = match self.token {
            Token::IDENT(ref id) => Ident(id.clone()),
            _ => self.unexpected_token(&self.token, Some("a identifier"))
        };
        self.bump();

        ident
    }

    fn parse_path(&mut self) -> IPath {
        let path = match self.token {
            Token::PATH(ref p) => IPath(p.clone()),
            _ => self.unexpected_token(&self.token, Some("a path"))
        };
        self.bump();

        path
    }

    // -------------------------------------------------------------------

    fn parse_address(&mut self) -> Option<u8> {
        self.expect(&Token::LBRACKET);
        let value = match self.token {
            Token::INTEGER(i) => Some(i),
            Token::UNDERSCORE => None,
            _ => self.unexpected_token(&self.token, Some("an address"))
        };
        self.bump();
        self.expect(&Token::RBRACKET);

        value
    }

    fn parse_label(&mut self) -> Ident {
        self.expect(&Token::COLON);

        self.parse_ident()
    }

    fn parse_constant(&mut self) -> Ident {
        self.expect(&Token::DOLLAR);
        self.parse_ident()
    }

    fn parse_argument(&mut self) -> ArgumentNode {
        let location = self.update_location();

        let arg = match self.token {
            Token::INTEGER(i) => { self.bump(); Argument::Literal(i) },
            Token::CHAR(c)    => { self.bump(); Argument::Char(c) },
            Token::LBRACKET   => Argument::Address(self.parse_address()),
            Token::DOLLAR     => Argument::Const(self.parse_constant()),
            Token::COLON      => Argument::Label(self.parse_label()),
            _ => self.unexpected_token(&self.token, Some("an argument"))
        };

        Argument::new(arg, location)
    }

    fn parse_macro_argument(&mut self) -> MacroArgumentNode {
        let location = self.update_location();

        if self.token_is_argument() {
            MacroArgument::new(MacroArgument::Argument(self.parse_argument()),
                               location)
        } else {
            MacroArgument::new(MacroArgument::Ident(self.parse_ident()),
                               location)
        }
    }

    // -------------------------------------------------------------------

    fn parse_include(&mut self) -> StatementNode {
        let location = self.update_location();

        self.bump();
        self.expect(&Token::IDENT(rcstr("import")));
        let path = self.parse_path();

        Statement::new(Statement::Include(path), location)
    }

    fn parse_label_def(&mut self) -> StatementNode {
        let location = self.update_location();

        let label = self.parse_ident();
        self.expect(&Token::COLON);

        Statement::new(Statement::Label(label), location)
    }

    fn parse_constant_def(&mut self) -> StatementNode {
        let location = self.update_location();

        let name = self.parse_constant();
        self.expect(&Token::EQ);
        let value = self.parse_argument();

        Statement::new(Statement::Const(name, value), location)
    }

    fn parse_operation(&mut self) -> StatementNode {
        let location = self.update_location();

        let mn = if let Token::MNEMONIC(mn) = self.token {
            Mnemonic(mn)
        } else {
            self.unexpected_token(&self.token, Some("a mnemonic"))
        };

        self.bump();

        let mut args = vec![];
        while self.token_is_argument() {
            args.push(self.parse_argument());
        }

        Statement::new(Statement::Operation(mn, args), location)
    }

    fn parse_macro(&mut self) -> StatementNode {
        let location = self.update_location();

        self.expect(&Token::AT);
        let name = self.parse_ident();

        self.expect(&Token::LPAREN);

        let mut args = vec![];
        if self.token != Token::RPAREN {
            loop {
                args.push(self.parse_macro_argument());
                if !self.eat(&Token::COMMA) {
                    break
                }
            }
        }
        self.expect(&Token::RPAREN);

        Statement::new(Statement::Macro(name, args), location)
    }

    fn parse_statement(&mut self) -> StatementNode {
        let stmt = match self.token {
            Token::HASH        => self.parse_include(),
            Token::DOLLAR      => self.parse_constant_def(),
            Token::IDENT(_)    => self.parse_label_def(),
            Token::MNEMONIC(_) => self.parse_operation(),
            Token::AT          => self.parse_macro(),

            ref tok => self.unexpected_token(tok, Some("a statement"))
        };

        stmt
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use assembler::ast::{
        AST, Statement, Argument, MacroArgument, Mnemonic, Ident, IPath,
        StatementNode, ArgumentNode, MacroArgumentNode
    };
    use assembler::lexer::{dummy_source, Token, Lexer};
    use assembler::lexer::Token::*;
    use assembler::util::rcstr;

    use super::*;

    fn parse<'a, T>(toks: Vec<Token>, f: |&mut Parser<'a>| -> T) -> T {
        f(&mut Parser::with_lexer(box toks as Box<Lexer>))
    }

    #[test]
    fn test_statements() {
        assert_eq!(
            parse(
                vec![HASH, IDENT(rcstr("import")), PATH(rcstr("as/d")),
                     MNEMONIC(from_str("HALT").unwrap())],
                |p| p.parse()
            ),
            vec![
                Statement::new(
                    Statement::Include(
                        IPath::from_str("as/d")
                    ),
                    dummy_source()
                ),
                Statement::new(
                    Statement::Operation(
                        Mnemonic(from_str("HALT").unwrap()),
                        vec![]
                    ),
                    dummy_source()
                )
            ]
        )
    }

    #[test]
    fn test_include() {
        assert_eq!(
            parse(vec![HASH, IDENT(rcstr("import")), PATH(rcstr("as/d"))],
                  |p| p.parse_statement()),
            Statement::new(
                Statement::Include(
                    IPath::from_str("as/d")
                ),
                dummy_source()
            )
        )
    }

    #[test]
    fn test_label_def() {
        assert_eq!(
            parse(vec![IDENT(rcstr("lbl")), COLON],
                  |p| p.parse_statement()),
            Statement::new(
                Statement::Label(
                    Ident::from_str("lbl")
                ),
                dummy_source()
            )
        )
    }

    #[test]
    fn test_const_def() {
        assert_eq!(
            parse(vec![DOLLAR, IDENT(rcstr("c")), EQ, INTEGER(0)],
                  |p| p.parse_statement()),
            Statement::new(
                Statement::Const(
                    Ident::from_str("c"),
                    Argument::new(
                        Argument::Literal(0),
                        dummy_source()
                    )
                ),
                dummy_source()
            )
        )
    }

    #[test]
    fn test_operation() {
        assert_eq!(
            parse(vec![MNEMONIC(from_str("MOV").unwrap()), INTEGER(0)],
                  |p| p.parse_statement()),
            Statement::new(
                Statement::Operation(
                    Mnemonic(from_str("MOV").unwrap()),
                    vec![
                        Argument::new(
                            Argument::Literal(0),
                            dummy_source()
                        )
                    ]
                ),
                dummy_source()
            )
        )
    }

    #[test]
    fn test_macro() {
        assert_eq!(
            parse(vec![AT, IDENT(rcstr("macro")),
                       LPAREN, INTEGER(0), COMMA, INTEGER(0), RPAREN],
                  |p| p.parse_statement()),
            Statement::new(
                Statement::Macro(
                    Ident::from_str("macro"),
                    vec![
                        MacroArgument::new(
                            MacroArgument::Argument(
                                Argument::new(
                                    Argument::Literal(0),
                                    dummy_source()
                                )
                            ),
                            dummy_source()
                        ),
                        MacroArgument::new(
                            MacroArgument::Argument(
                                Argument::new(
                                    Argument::Literal(0),
                                    dummy_source()
                                )
                            ),
                            dummy_source()
                        )
                    ]
                ),
                dummy_source()
            )
        )
    }

    #[test]
    fn test_literal() {
        assert_eq!(
            parse(vec![INTEGER(0)],
                  |p| p.parse_argument()),
            Argument::new(
                Argument::Literal(0),
                dummy_source()
            )
        )
    }

    #[test]
    fn test_address() {
        assert_eq!(
            parse(vec![LBRACKET, INTEGER(0), RBRACKET],
                  |p| p.parse_argument()),
            Argument::new(
                Argument::Address(Some(0)),
                dummy_source()
            )
        )
    }

    #[test]
    fn test_address_auto() {
        assert_eq!(
            parse(vec![LBRACKET, UNDERSCORE, RBRACKET],
                  |p| p.parse_argument()),
            Argument::new(
                Argument::Address(None),
                dummy_source()
            )
        )
    }

    #[test]
    fn test_const() {
        assert_eq!(
            parse(vec![DOLLAR, IDENT(rcstr("asd"))],
                  |p| p.parse_argument()),
            Argument::new(
                Argument::Const(
                    Ident::from_str("asd")
                ),
                dummy_source()
            )
        )
    }

    #[test]
    fn test_label() {
        assert_eq!(
            parse(vec![COLON, IDENT(rcstr("asd"))],
                  |p| p.parse_argument()),
            Argument::new(
                Argument::Label(
                    Ident::from_str("asd")
                ),
                dummy_source()
            )
        )
    }

    #[test]
    fn test_char() {
        assert_eq!(
            parse(vec![CHAR(0)],
                  |p| p.parse_argument()),
            Argument::new(
                Argument::Char(0),
                dummy_source()
            )
        )
    }

    #[test]
    fn test_macro_arg_arg() {
        assert_eq!(
            parse(vec![INTEGER(0)],
                  |p| p.parse_macro_argument()),
            MacroArgument::new(
                MacroArgument::Argument(
                    Argument::new(
                        Argument::Literal(0),
                        dummy_source()
                    )
                ),
                dummy_source()
            )
        )
    }

    #[test]
    fn test_macro_arg_ident() {
        assert_eq!(
            parse(vec![IDENT(rcstr("asd"))],
                  |p| p.parse_macro_argument()),
            MacroArgument::new(
                MacroArgument::Ident(
                    Ident::from_str("asd")
                ),
                dummy_source()
            )
        )
    }

    #[test]
    fn test_op_and_const() {
        assert_eq!(
            parse(vec![MNEMONIC(from_str("HALT").unwrap()),
                       DOLLAR, IDENT(rcstr("c")), EQ, INTEGER(0)],
                  |p| p.parse()),
            vec![
                Statement::new(
                    Statement::Operation(
                        Mnemonic(from_str("HALT").unwrap()),
                        vec![]
                    ),
                    dummy_source()
                ),
                Statement::new(
                    Statement::Const(
                        Ident::from_str("c"),
                        Argument::new(
                            Argument::Literal(0),
                            dummy_source()
                        )
                    ),
                    dummy_source()
                )
            ]
        )
    }
}