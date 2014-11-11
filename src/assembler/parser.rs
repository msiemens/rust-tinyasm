use std;
use std::rc::Rc;

use assembler::ast::*;
use assembler::lexer::*;
use assembler::util::fatal;


pub struct Parser<'a> {
    token: Token,
    lookahead: Token,
    lexer: Lexer<'a>
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, file: &'a str) -> Parser<'a> {
        let mut lx = Lexer::new(source, file);

        Parser {
            token: lx.next_token(),
            lookahead: PLACEHOLDER,
            lexer: lx
        }
    }

    fn unexpected_token(&self, tok: &Token) -> ! {
        fatal(format!("unexpected token: `{}`", tok),
              self.lexer.get_source());
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
                  self.lexer.get_source())
        }
    }

    pub fn parse(&mut self) -> Vec<Statement> {
        let mut ast = vec![];

        while !self.lexer.is_eof() {
            ast.push(self.parse_statement());
            debug!("so far: {}", ast);
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
            _ => self.unexpected_token(&self.token)
        };
        self.bump();

        ident
    }

    fn parse_path(&mut self) -> Path {
        let path = match self.token {
            PATH(ref p) => Path(p.clone()),
            _ => self.unexpected_token(&self.token)
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
            _ => self.unexpected_token(&self.token)
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
            _ => self.unexpected_token(&self.token)
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
            _ => self.unexpected_token(&self.token)
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

            ref tok => self.unexpected_token(tok)
        }
    }
}