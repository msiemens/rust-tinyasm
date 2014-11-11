use term;
use std;

use assembler::SourceLocation;

pub fn fatal(msg: String, source: SourceLocation) -> ! {
    let mut t = term::stdout().unwrap();

    t.fg(term::color::RED).unwrap();
    (write!(t, "Error ")).unwrap();

    t.reset().unwrap();
    (write!(t, "in {}: ", source)).unwrap();

    t.fg(term::color::YELLOW).unwrap();
    (write!(t, "{}\n", msg)).unwrap();

    t.reset().unwrap();

    std::io::stdio::set_stderr(box std::io::util::NullWriter);
    panic!();
}