use assembler::ast::AST;


mod imports;
mod subroutines;
mod auto_address;
mod constants;
mod labels;


pub fn expand_syntax_extensions(ast: &mut AST) {
    imports::expand(ast);
    subroutines::expand(ast);
    auto_address::expand(ast);
    constants::expand(ast);
    labels::expand(ast);
}
