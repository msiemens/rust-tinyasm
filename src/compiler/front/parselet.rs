use std::collections::HashMap;
use std::rc::Rc;
use ast::*;
use front::tokens::{Token, TokenType, Keyword};
use front::parser::Parser;


pub fn get_prefix_parselets() -> Rc<HashMap<TokenType, Box<PrefixParselet>>> {
    macro_rules! register(
        ($map:expr, $token:expr, $parselet:expr) => (
            $map.insert($token, Box::new($parselet) as Box<PrefixParselet>)
        )
    );

    thread_local! {
        static PREFIX_PARSELETS: Rc<HashMap<TokenType, Box<PrefixParselet>>> = {
            let mut map = HashMap::new();
            register!(map, TokenType::Ident, IdentParselet);
            register!(map, TokenType::Bool,  LiteralParselet);
            register!(map, TokenType::Int,   LiteralParselet);
            register!(map, TokenType::Char,  LiteralParselet);
            register!(map, TokenType::True,  LiteralParselet);
            register!(map, TokenType::False, LiteralParselet);
            register!(map, TokenType::Not,   PrefixOperatorParselet);
            register!(map, TokenType::Sub,   PrefixOperatorParselet);
            register!(map, TokenType::LParen, GroupParselet);

            Rc::new(map)
        }
    };

    PREFIX_PARSELETS.with(|o| o.clone())
}

pub trait PrefixParselet {
    fn parse(&self, parser: &mut Parser, token: Token) -> Node<Expression>;
    fn name(&self) -> &'static str;
}

pub struct IdentParselet;
impl PrefixParselet for IdentParselet {
    fn parse(&self, parser: &mut Parser, token: Token) -> Node<Expression> {
        let ident = match token {
            Token::Ident(id) => id,
            _ => parser.unexpected_token(Some("an identifier"))
        };

        Node::new(Expression::Variable { name: ident })
    }

    fn name(&self) -> &'static str { "IdentParselet" }
}

pub struct LiteralParselet;
impl PrefixParselet for LiteralParselet {
    fn parse(&self, parser: &mut Parser, token: Token) -> Node<Expression> {
        let value = match token {
            Token::Int(i) => Value::Int(i),
            Token::Char(c) => Value::Char(c),
            Token::Keyword(Keyword::True) => Value::Bool(true),
            Token::Keyword(Keyword::False) => Value::Bool(false),
            _ => parser.unexpected_token(Some("a literal"))
        };

        Node::new(Expression::Literal {
            val: value
        })
    }

    fn name(&self) -> &'static str { "LiteralParselet" }
}

pub struct PrefixOperatorParselet;
impl PrefixParselet for PrefixOperatorParselet {
    fn parse(&self, parser: &mut Parser, token: Token) -> Node<Expression> {
        let operand = parser.parse_expression();
        let op = match token {
            Token::UnOp(op) => op,
            Token::BinOp(BinOp::Sub) => UnOp::Neg,  // FIXME
            _ => parser.unexpected_token(Some("an unary operator"))
        };

        Node::new(Expression::Prefix { op: op, item: Box::new(operand) })
    }

    fn name(&self) -> &'static str { "PrefixOperatorParselet" }
}

pub struct GroupParselet;
impl PrefixParselet for GroupParselet {
    fn parse(&self, parser: &mut Parser, token: Token) -> Node<Expression> {
        let expr = parser.parse_expression();
        parser.expect(Token::RParen);

        expr
    }

    fn name(&self) -> &'static str { "GroupParselet" }
}


pub fn get_infix_parselets() -> Rc<HashMap<TokenType, Box<InfixParselet>>> {
    macro_rules! register(
        ($map:expr, $token:expr, $parselet:expr) => (
            $map.insert($token, Box::new($parselet) as Box<InfixParselet>)
        )
    );

    thread_local! {
        static INFIX_PARSELETS: Rc<HashMap<TokenType, Box<InfixParselet>>> = {
            let mut map = HashMap::new();
            register!(map, TokenType::Add,    BinaryOperatorParselet);
            register!(map, TokenType::Sub,    BinaryOperatorParselet);
            register!(map, TokenType::Mul,    BinaryOperatorParselet);
            register!(map, TokenType::Div,    BinaryOperatorParselet);
            register!(map, TokenType::Mod,    BinaryOperatorParselet);
            register!(map, TokenType::Pow,    BinaryOperatorParselet);
            register!(map, TokenType::And,    BinaryOperatorParselet);
            register!(map, TokenType::Or,     BinaryOperatorParselet);
            register!(map, TokenType::BitXor, BinaryOperatorParselet);
            register!(map, TokenType::BitAnd, BinaryOperatorParselet);
            register!(map, TokenType::BitOr,  BinaryOperatorParselet);
            register!(map, TokenType::Shl,    BinaryOperatorParselet);
            register!(map, TokenType::Shr,    BinaryOperatorParselet);
            register!(map, TokenType::Lt,     BinaryOperatorParselet);
            register!(map, TokenType::Le,     BinaryOperatorParselet);
            register!(map, TokenType::Ne,     BinaryOperatorParselet);
            register!(map, TokenType::Ge,     BinaryOperatorParselet);
            register!(map, TokenType::Gt,     BinaryOperatorParselet);
            register!(map, TokenType::EqEq,   BinaryOperatorParselet);
            register!(map, TokenType::Eq,     AssignParselet);
            register!(map, TokenType::LParen, CallParselet);

            Rc::new(map)
        }
    };

    INFIX_PARSELETS.with(|o| o.clone())
}

pub trait InfixParselet {
    fn parse(&self, parser: &mut Parser, left: Node<Expression>, token: Token) -> Node<Expression>;
    fn name(&self) -> &'static str;
}

pub struct BinaryOperatorParselet;
impl InfixParselet for BinaryOperatorParselet {
    fn parse(&self, parser: &mut Parser, left: Node<Expression>, token: Token) -> Node<Expression> {
        let op = match token {
            Token::BinOp(op) => op,
            _ => parser.unexpected_token(Some("a binary operator"))
        };

        if parser.token == Token::Eq {
            parser.bump();
            let right = parser.parse_expression();
            Node::new(Expression::AssignOp {
                op: op,
                lhs: Box::new(left),
                rhs: Box::new(right)
            })
        } else {
            let right = parser.parse_expression();
            Node::new(Expression::Infix {
                op: op,
                lhs: Box::new(left),
                rhs: Box::new(right)
            })
        }
    }

    fn name(&self) -> &'static str { "BinaryOperatorParselet" }
}

pub struct AssignParselet;
impl InfixParselet for AssignParselet {
    fn parse(&self, parser: &mut Parser, left: Node<Expression>, token: Token) -> Node<Expression> {
        let right = parser.parse_expression();

        /*match *left {
            Expression::Variable {..} => {},
            _ => panic!("not a name")
        }*/

        Node::new(Expression::Assign {
            lhs: Box::new(left),
            rhs: Box::new(right)
        })
    }

    fn name(&self) -> &'static str { "AssignParselet" }
}

pub struct CallParselet;
impl InfixParselet for CallParselet {
    fn parse(&self, parser: &mut Parser, left: Node<Expression>, token: Token) -> Node<Expression> {
        let mut args = vec![];

        /*match *left {
            Expression::Variable {..} => {},
            _ => panic!("not a name")
        }*/

        while parser.token != Token::RParen {
            args.push(parser.parse_expression());
            if !parser.eat(Token::Comma) {
                break
            }
        }

        parser.expect(Token::RParen);

        Node::new(Expression::Call {
            func: Box::new(left),
            args: args
        })
    }

    fn name(&self) -> &'static str { "CallParselet" }
}