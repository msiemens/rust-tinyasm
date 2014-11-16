use assembler::ast::Statement;


mod imports;
mod subroutines;
mod auto_address;
mod constants;
mod labels;


pub fn expand_syntax_extensions(ast: &mut Vec<Statement>) {
    imports::expand(ast);
    subroutines::expand(ast);
    auto_address::expand(ast);
    constants::expand(ast);
    labels::expand(ast);
}
