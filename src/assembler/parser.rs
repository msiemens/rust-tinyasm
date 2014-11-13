use std;
use std::rc::Rc;

use assembler::ast::*;
use assembler::lexer::*;
use assembler::util::fatal;


pub struct Parser<'a> {
    token: Token,
    lookahead: Token,
    lexer: Box<Lexer + 'a>
}

impl<'a> Parser<'a> {
    #[cfg(not(test))]
    pub fn new(source: &'a str, file: &'a str) -> Parser<'a> {
        Parser::with_lexer(box FileLexer::new(source, file))
    }

    pub fn with_lexer(lx: Box<Lexer + 'a>) -> Parser<'a> {
        let mut lx = lx;

        Parser {
            token: lx.next_token(),
            lookahead: PLACEHOLDER,
            lexer: lx
        }
    }

    fn unexpected_token(&self, tok: &Token, expected: Option<&'static str>) -> ! {
        match expected {
            Some(ex) => fatal(format!("unexpected token: `{}`, expected {}", tok, ex),
                              &self.lexer.get_source()),
            None => fatal(format!("unexpected token: `{}`", tok),
                          &self.lexer.get_source())
        }
    }

    fn bump(&mut self) {
        let next = if self.lookahead == PLACEHOLDER {
            self.lexer.next_token()
        } else {
            std::mem::replace(&mut self.lookahead, PLACEHOLDER)
        };

        std::mem::replace(&mut self.token, next);
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
            fatal(format!("expected `{}`, found `{}`", tok, self.token),
                  &self.lexer.get_source())
        }
    }

    pub fn parse(&mut self) -> Vec<Statement> {
        let mut ast = vec![];

        while self.token != EOF {
            ast.push(self.parse_statement());
        }


        ast
    }

    // -------------------------------------------------------------------

    fn token_is_argument(&self) -> bool {
        match self.token {
            INTEGER(_) | CHAR(_) | LBRACKET | DOLLAR | COLON => true,
            _ => false
        }
    }

    // -------------------------------------------------------------------

    fn parse_ident(&mut self) -> Ident {
        let ident = match self.token {
            IDENT(ref id) => Ident(id.clone()),
            _ => self.unexpected_token(&self.token, Some("a identifier"))
        };
        self.bump();

        ident
    }

    fn parse_path(&mut self) -> Path {
        let path = match self.token {
            PATH(ref p) => Path(p.clone()),
            _ => self.unexpected_token(&self.token, Some("a path"))
        };
        self.bump();

        path
    }

    // -------------------------------------------------------------------

    fn parse_address(&mut self) -> Option<u8> {
        self.expect(&LBRACKET);
        let value = match self.token {
            INTEGER(i) => Some(i),
            UNDERSCORE => None,
            _ => self.unexpected_token(&self.token, Some("an address"))
        };
        self.bump();
        self.expect(&RBRACKET);

        value
    }

    fn parse_label(&mut self) -> Ident {
        self.expect(&COLON);

        self.parse_ident()
    }

    fn parse_constant(&mut self) -> Ident {
        self.expect(&DOLLAR);
        self.parse_ident()
    }

    fn parse_argument(&mut self) -> Argument {
        let arg = match self.token {
            INTEGER(i) => { self.bump(); ArgumentLiteral(i) },
            CHAR(c) => { self.bump(); ArgumentChar(c) },
            LBRACKET => ArgumentAddress(self.parse_address()),
            DOLLAR => ArgumentConst(self.parse_constant()),
            COLON => ArgumentLabel(self.parse_label()),
            _ => self.unexpected_token(&self.token, Some("an argument"))
        };

        Argument::new(arg, self.lexer.get_source())
    }

    fn parse_macro_argument(&mut self) -> MacroArgument {
        if self.token_is_argument() {
            MacroArgument::new(MacroArgArgument(self.parse_argument()),
                               self.lexer.get_source())
        } else {
            MacroArgument::new(MacroArgIdent(self.parse_ident()),
                               self.lexer.get_source())
        }
    }

    // -------------------------------------------------------------------

    fn parse_include(&mut self) -> Statement {
        self.bump();
        self.expect(&IDENT(Rc::new("import".into_string())));
        let path = self.parse_path();

        Statement::new(StatementInclude(path), self.lexer.get_source())
    }

    fn parse_label_def(&mut self) -> Statement {
        let label = self.parse_ident();
        self.expect(&COLON);

        Statement::new(StatementLabel(label), self.lexer.get_source())
    }

    fn parse_constant_def(&mut self) -> Statement {
        let name = self.parse_constant();
        self.expect(&EQ);
        let value = self.parse_argument();

        Statement::new(StatementConst(name, value), self.lexer.get_source())
    }

    fn parse_operation(&mut self) -> Statement {
        let mn = match self.token {
            MNEMONIC(mn) => Mnemonic(mn),
            _ => self.unexpected_token(&self.token, Some("a mnemonic"))
        };
        self.bump();

        let mut args = vec![];
        while self.token_is_argument() {
            args.push(self.parse_argument());
        }

        Statement::new(StatementOperation(mn, args), self.lexer.get_source())
    }

    fn parse_macro(&mut self) -> Statement {
        self.expect(&AT);
        let name = self.parse_ident();
        self.expect(&LPAREN);

        let mut args = vec![];
        if self.token != RPAREN {
            loop {
                args.push(self.parse_macro_argument());
                if !self.eat(&COMMA) {
                    break
                }
            }
        }
        self.expect(&RPAREN);

        Statement::new(StatementMacro(name, args), self.lexer.get_source())
    }

    fn parse_statement(&mut self) -> Statement {
        match self.token {
            HASH => self.parse_include(),
            DOLLAR => self.parse_constant_def(),
            IDENT(_) => self.parse_label_def(),
            MNEMONIC(_) => self.parse_operation(),
            AT => self.parse_macro(),

            ref tok => self.unexpected_token(tok, Some("a statement"))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use assembler::dummy_source;
    use assembler::lexer::*;
    use assembler::ast::*;

    use super::*;

    fn parse<'a, T>(toks: Vec<Token>, f: |&mut Parser<'a>| -> T) -> T {
        f(&mut Parser::with_lexer(box toks as Box<Lexer>))
    }

    #[test]
    fn test_statements() {
        assert_eq!(
            parse(
                vec![HASH, IDENT(rcstr!("import")), PATH(rcstr!("as/d")),
                     MNEMONIC(from_str("HALT").unwrap())],
                |p| p.parse()
            ),
            vec![
                Statement::new(
                    StatementInclude(
                        Path(rcstr!("as/d"))
                    ),
                    dummy_source()
                ),
                Statement::new(
                    StatementOperation(
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
            parse(vec![HASH, IDENT(rcstr!("import")), PATH(rcstr!("as/d"))],
                  |p| p.parse_statement()),
            Statement::new(
                StatementInclude(
                    Path(rcstr!("as/d"))
                ),
                dummy_source()
            )
        )
    }

    #[test]
    fn test_label_def() {
        assert_eq!(
            parse(vec![IDENT(rcstr!("lbl")), COLON],
                  |p| p.parse_statement()),
            Statement::new(
                StatementLabel(
                    Ident(rcstr!("lbl"))
                ),
                dummy_source()
            )
        )
    }

    #[test]
    fn test_const_def() {
        assert_eq!(
            parse(vec![DOLLAR, IDENT(rcstr!("c")), EQ, INTEGER(0)],
                  |p| p.parse_statement()),
            Statement::new(
                StatementConst(
                    Ident(rcstr!("c")),
                    Argument::new(
                        ArgumentLiteral(0),
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
                StatementOperation(
                    Mnemonic(from_str("MOV").unwrap()),
                    vec![
                        Argument::new(
                            ArgumentLiteral(0),
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
            parse(vec![AT, IDENT(rcstr!("macro")),
                       LPAREN, INTEGER(0), COMMA, INTEGER(0), RPAREN],
                  |p| p.parse_statement()),
            Statement::new(
                StatementMacro(
                    Ident(rcstr!("macro")),
                    vec![
                        MacroArgument::new(
                            MacroArgArgument(
                                Argument::new(
                                    ArgumentLiteral(0),
                                    dummy_source()
                                )
                            ),
                            dummy_source()
                        ),
                        MacroArgument::new(
                            MacroArgArgument(
                                Argument::new(
                                    ArgumentLiteral(0),
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
                ArgumentLiteral(0),
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
                ArgumentAddress(Some(0)),
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
                ArgumentAddress(None),
                dummy_source()
            )
        )
    }

    #[test]
    fn test_const() {
        assert_eq!(
            parse(vec![DOLLAR, IDENT(rcstr!("asd"))],
                  |p| p.parse_argument()),
            Argument::new(
                ArgumentConst(
                    Ident(rcstr!("asd"))
                ),
                dummy_source()
            )
        )
    }

    #[test]
    fn test_label() {
        assert_eq!(
            parse(vec![COLON, IDENT(rcstr!("asd"))],
                  |p| p.parse_argument()),
            Argument::new(
                ArgumentLabel(
                    Ident(rcstr!("asd"))
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
                ArgumentChar(0),
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
                MacroArgArgument(
                    Argument::new(
                        ArgumentLiteral(0),
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
            parse(vec![IDENT(rcstr!("asd"))],
                  |p| p.parse_macro_argument()),
            MacroArgument::new(
                MacroArgIdent(
                    Ident(rcstr!("asd"))
                ),
                dummy_source()
            )
        )
    }
}