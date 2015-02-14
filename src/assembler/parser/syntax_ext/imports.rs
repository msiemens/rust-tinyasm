//! A syntax extension for imports
//!
//! # Example:
//!
//! `a.asm`:
//!
//! ```
//! APRINT '!'
//! ```
//!
//! `b.asm`:
//!
//! ```
//! #import <a.asm>
//! HALT
//! ```
//!
//! Results in:
//!
//! ```
//! APRINT '!'
//! HALT
//! ```
//!
//! # Note:
//!
//! A file will be imported only once. Circular imports are not allowed.

use std::ffi::AsOsStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use assembler::util::fatal;
use assembler::parser::ast::{Program, Statement};
use assembler::parser::Parser;


pub fn expand(source: &mut Program) {
    let mut last_file = None;

    // We use a indexed iteration here because we'll modify the source as we iterate
    // over it
    let mut i = 0;
    while i < source.len() {
        // Process import statements
        let mut included_source = if let Statement::Include(ref include) = source[i].value {
            // Get path to include
            let path = Path::new(&*source[i].location.filename);

            let dir = Path::new(path.parent().unwrap_or(Path::new(".")));
            let to_include = dir.join(&*include.as_str());

            // Forbid circular imports
            if last_file == Some(to_include.clone()) {
                fatal!("circular import of {}", to_include.display(); source[i]);
            }
            last_file = Some(to_include.clone());

            // Read source file
            let mut file = File::open(&to_include).unwrap_or_else(|e| {
                fatal!("cannot read {}: {}", to_include.display(), e; source[i]);
            });

            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap_or_else(|e| {
                fatal!("cannot read {}: {}", to_include.display(), e; source[i]);
            });

            // Parse it
            let mut parser = Parser::new(&contents, to_include.as_os_str().to_str().unwrap());
            parser.parse()
        } else {
            i += 1;
            continue
        };

        // Remove the `#import <...>` statement
        source.remove(i);

        // Insert the new source into the current one
        for j in range(0, included_source.len()) {
            source.insert(i + j, included_source.remove(0));
        }
    }
}