use std::old_io;
use std::rc::Rc;
use ansi_term::Colour::{Red, Yellow};
use assembler::parser::SourceLocation;


// FIXME: Use String intering with a table and something like `struct IString(u32)`
pub type SharedString = Rc<String>;

pub fn rcstr<'a>(s: &'a str) -> SharedString {
    Rc::new(String::from_str(s))
}


#[macro_export]
macro_rules! impl_to_string(
    ($cls:ident: $fmt:expr, $( $args:ident ),*) => (
        impl fmt::Debug for $cls {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, $fmt, $( self.$args ),*)
            }
        }

        impl fmt::Display for $cls {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    )
);


#[macro_export]
macro_rules! overflow_check(
    ($val:expr, $stmt:expr) => (
        if $val > 255 {
            warn!("overflow: {} > 255", $val; $stmt);
            ($val as u32 % !(0 as ::machine::WordSize) as u32) as ::machine::WordSize
        }
        else { $val as ::machine::WordSize }
    )
);


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

pub fn fatal(msg: String, source: &SourceLocation) -> ! {
    println!("{} in {}: {}", Red.paint("Error"), source, msg);

    old_io::stdio::set_stderr(Box::new(old_io::util::NullWriter));
    panic!();
}


#[macro_export]
macro_rules! warn(
    ($msg:expr, $($args:expr),* ; $stmt:expr ) => {
        {
            ::assembler::util::warn(format!($msg, $($args),*), &$stmt.location)
        }
    }
);

pub fn warn(msg: String, source: &SourceLocation) {
    println!("{} in {}: {}", Yellow.paint("Warning"), source, msg);
}