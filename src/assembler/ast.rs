use super::instructions::Instructions;


#[deriving(Show)]
pub struct Ident(pub String);

#[deriving(Show)]
pub struct Mnemonic(pub Instructions);

#[deriving(Show)]
pub struct Path(pub String);

// See: http://doc.rust-lang.org/syntax/ast/enum.Item_.html
// See: http://doc.rust-lang.org/syntax/ast/enum.Expr_.html