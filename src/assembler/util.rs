#![macro_escape]
use term;
use std;
use std::rc::Rc;

use assembler::lexer::SourceLocation;


pub type SharedString = Rc<String>;

pub fn rcstr(s: &'static str) -> SharedString {
    Rc::new(s.into_string())
}

pub fn rcstring(s: String) -> SharedString {
    Rc::new(s)
}


macro_rules! overflow_check(
    ($val:expr @ $stmt:expr) => (
        if $val > 255 { warn!("Overflow: {} > 255", $val @ $stmt) }
    )
)


macro_rules! fatal(
    ($msg:expr, $($args:expr),* @ $stmt:expr) => {
        fatal(format!($msg, $($args),*), &$stmt.location)
    };

    ($msg:expr @ $stmt:expr) => {
        fatal($msg.into_string(), &$stmt.location)
    };
)

pub fn fatal(msg: String, source: &SourceLocation) -> ! {
    let mut t = term::stdout().unwrap();

    t.fg(term::color::RED).unwrap();
    (write!(t, "Error ")).unwrap();

    t.reset().unwrap();
    (write!(t, "in {}: ", source)).unwrap();
    (write!(t, "{}\n", msg)).unwrap();

    t.reset().unwrap();

    std::io::stdio::set_stderr(box std::io::util::NullWriter);
    panic!();
}


macro_rules! warn(
    ($msg:expr, $($args:expr),* @ $stmt:expr ) => {
        warn(format!($msg, $($args),*), &$stmt.location)
    }
)

pub fn warn(msg: String, source: &SourceLocation) {
    let mut t = term::stdout().unwrap();

    t.fg(term::color::YELLOW).unwrap();
    (write!(t, "Warning ")).unwrap();

    t.reset().unwrap();
    (write!(t, "in {}: ", source)).unwrap();
    (write!(t, "{}\n", msg)).unwrap();

    t.reset().unwrap();
}