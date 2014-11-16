use std::io::File;

use assembler::ast::*;
use assembler::parser::Parser;
use assembler::util::fatal;


pub fn expand(ast: &mut Vec<Statement>) {
    // TODO: Implement imports, cf https://github.com/msiemens/TINY.ASM/blob/master/preprocessor/imports.py

    // Pass 2: Replace label usages
    let mut i = 0;
    while i < ast.len() {
        let included_ast = match ast[i].node {
            StatementInclude(ref include) => {
                let path = Path::new(&*ast[i].location.filename);
                let dir = Path::new(path.dirname());
                let to_include = dir.join(include.as_str()[]);

                // TODO: Check for multiple/circular includes

                let contents = File::open(&to_include)
                    .read_to_string()
                    .unwrap_or_else(|e| {
                        fatal!("Cannot read {}: {}", to_include.display(), e
                               @ ast[i]);
                    });
                let mut parser = Parser::new(contents[], to_include.as_str().unwrap());

                Some(parser.parse())

                // TODO: Include new ast into ast
            },
            _ => None
        };

        if included_ast.is_some() {
            let mut included_ast = included_ast.unwrap();

            for j in range(0, included_ast.len()) {
                ast.insert(i + j, included_ast.remove(0).unwrap());
            }
        }

        i += 1;
    }
}