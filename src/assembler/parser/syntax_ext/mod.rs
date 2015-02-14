use assembler::parser::ast::Program;

mod imports;
mod subroutines;
mod auto_address;
mod constants;
mod labels;

pub fn expand_syntax_extensions(source: &mut Program) {
    imports::expand(source);
    subroutines::expand(source);
    auto_address::expand(source);
    constants::expand(source);
    labels::expand(source);
}
