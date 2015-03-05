use ansi_term::Colour::{Red, Yellow};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::old_io;
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;
use std::str::StrExt;
use ast::Ident;

/// Read a file and return it's contents
pub fn read_file(input_path: &str) -> String {
    let mut file = match File::open(&Path::new(input_path)) {
        Ok(f) => f,
        Err(err) => panic!("Can't open {}: {}", input_path, err)
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(contents) => contents,
        Err(_) => panic!("Can't read {}", input_path)
    };

    contents
}


// --- Warnings and errors ------------------------------------------------------

#[macro_export]
macro_rules! fatal(
    ($msg:expr, $($args:expr),* ; $stmt:expr) => {
        {
            use assembler::util::fatal;
            fatal(format!($msg, $($args),*), &$stmt.location)
        }
    };

    ($msg:expr ; $stmt:expr) => {
        {
            use std::borrow::ToOwned;
            ::assembler::util::fatal($msg.to_owned(), &$stmt.location)
        }
    };
);

pub fn fatal(msg: String, source: usize) -> ! {
    println!("{} in line {}: {}", Red.paint("Error"), source, msg);

    old_io::stdio::set_stderr(Box::new(old_io::util::NullWriter));
    panic!();
}


#[macro_export]
macro_rules! warn(
    ($msg:expr, $($args:expr),* ; $stmt:expr ) => {
        ::assembler::util::warn(format!($msg, $($args),*), &$stmt.location)
    }
);

pub fn warn(msg: String, source: usize) {
    println!("{} in line {}: {}", Yellow.paint("Warning"), source, msg);
}


// --- String interner ----------------------------------------------------------
// Inspired by: http://doc.rust-lang.org/src/syntax/util/interner.rs.html

#[derive(Clone, PartialEq, Hash, PartialOrd)]
pub struct RcStr {
    string: Rc<String>,
}

impl RcStr {
    pub fn new(string: &str) -> RcStr {
        RcStr {
            string: Rc::new(string.to_string()),
        }
    }
}

impl Eq for RcStr {}

impl Ord for RcStr {
    fn cmp(&self, other: &RcStr) -> Ordering {
        self[..].cmp(&other[..])
    }
}

impl fmt::Debug for RcStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::fmt::Debug;
        self[..].fmt(f)
    }
}

impl fmt::Display for RcStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::fmt::Display;
        self[..].fmt(f)
    }
}

impl Borrow<str> for RcStr {
    fn borrow(&self) -> &str {
        &self.string[..]
    }
}

impl Deref for RcStr {
    type Target = str;

    fn deref(&self) -> &str { &self.string[..] }
}


pub struct Interner {
    map: RefCell<HashMap<RcStr, Ident>>,
    vec: RefCell<Vec<RcStr>>
}

impl Interner {
    pub fn new() -> Interner {
        Interner {
            map: RefCell::new(HashMap::new()),
            vec: RefCell::new(Vec::new())
        }
    }

    pub fn intern(&self, val: &str) -> Ident {
        let mut map = self.map.borrow_mut();
        let mut vec = self.vec.borrow_mut();

        if let Some(&idx) = map.get(val) {
            return idx;
        }

        let idx = Ident(vec.len());
        let val = RcStr::new(val);
        map.insert(val.clone(), idx);
        vec.push(val);

        idx
    }

    pub fn get(&self, ident: Ident) -> RcStr {
        let Ident(idx) = ident;
        self.vec.borrow()[idx].clone()
    }
}