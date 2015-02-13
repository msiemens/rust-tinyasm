use std::fs::File;
use std::io::Read;

use assembler::ast::{AST, Statement};
use assembler::parser::Parser;
use assembler::util::fatal;


pub fn expand(ast: &mut AST) {
    // TODO: Implement imports, cf https://github.com/msiemens/TINY.ASM/blob/master/preprocessor/imports.py
    let mut i = 0;
    let mut last_file = None;

    while i < ast.len() {
        let mut included_ast = if let Statement::Include(ref include) = ast[i].value {
            let path = Path::new(&*ast[i].location.filename);
            let dir = Path::new(path.dirname());
            let to_include = dir.join(&*include.as_str());

            if last_file == Some(to_include.clone()) {
                fatal!("circular import of {}", to_include.display(); ast[i]);
            }

            last_file = Some(to_include.clone());

            let mut contents = String::new();
            File::open(&to_include)
                .unwrap_or_else(|e| {
                    fatal!("cannot read {}: {}", to_include.display(), e; ast[i]);
                })
                .read_to_string(&mut contents)
                .unwrap_or_else(|e| {
                    fatal!("cannot read {}: {}", to_include.display(), e; ast[i]);
                });
            let mut parser = Parser::new(&*contents, to_include.as_str().unwrap());

            parser.parse()

        } else {
            i += 1;
            continue
        };

        ast.remove(i);

        for j in range(0, included_ast.len()) {
            ast.insert(i + j, included_ast.remove(0));
        }
    }
}