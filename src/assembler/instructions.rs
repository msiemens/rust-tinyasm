use std::ascii::AsciiExt;
use std::collections::HashMap;
use std::str::FromStr;

use self::Instructions::*;
use self::ArgumentType::*;


#[deriving(PartialEq, Show, Clone)]
pub enum ArgumentType {
    Address,
    Literal
}

impl FromStr for ArgumentType {
    fn from_str(s: &str) -> Option<ArgumentType> {
        match s.char_at(0) {
            '[' => Some(Address),
            _   => Some(Literal)
        }
    }
}


macro_rules! make_instructions(
    ( $( $op:ident ),* ) => {

        #[deriving(Clone, PartialEq, Eq, Show, Hash)]
        pub enum Instructions {
            $( $op ),*
        }

        impl FromStr for Instructions {
            fn from_str(s: &str) -> Option<Instructions> {
                match s.to_ascii_upper()[] {
                    $(
                        stringify!($op) => Some($op),
                    )*
                    _ => None
                }
            }
        }

    }
);

make_instructions!(
    AND, OR, XOR, NOT,              // Logic
    MOV,                            // Memory
    RANDOM, ADD, SUB,               // Math
    JMP, JZ, JEQ, JLS, JGT, HALT,   // Control
    APRINT, DPRINT, AREAD           // IO
);


#[deriving(Show, Clone)]
struct Operation {
    pub opcode: u8,
    pub args: Vec<ArgumentType>
}

pub type InstructionClass = Vec<Operation>;
pub type InstructionSet = HashMap<Instructions, InstructionClass>;


macro_rules! instructions (
    ( $opcode:expr ; ) => (
        seq!{
            Operation { opcode: $opcode, args: seq![] },
        }
    );

    ( $( $opcode:expr : $( $args:ident ),* ; )* ) => (
        seq!{
            $(
                Operation { opcode: $opcode, args: seq![ $( $args ),* ] },
            )*
        }
    )
);

lazy_static! {
    pub static ref INSTRUCTIONS: InstructionSet = {
        seq!{
            AND => instructions!{
                // M[a] = M[a] bit-wise and M[b]
                // opcode | a | b
                0x00: Address, Address;
                0x01: Address, Literal;
            },
            OR => instructions!{
                // M[a] = M[a] bit-wise or M[b]
                // opcode | a | b
                0x02: Address, Address;
                0x03: Address, Literal;
            },
            XOR => instructions!{
                // M[a] = M[a] bitwise xor M[b]
                // opcode | a | b
                0x04: Address, Address;
                0x05: Address, Literal;
            },
            NOT => instructions!{
                // M[a] = bit-wise not M[a]
                // opcode | a
                0x06: Address;
            },
            MOV => instructions!{
                // M[a] = M[b], or the Literal-set M[a] = b
                // opcode | a | b:
                0x07: Address, Address;
                0x08: Address, Literal;
            },
            RANDOM => instructions!{
                // M[a] = random value (0 to 25; equal probability distribution)
                // opcode | a:
                0x09: Address;
            },
            ADD => instructions!{
                // M[a] = M[a] + b; no overflow support
                // opcode | a | b:
                0x0A: Address, Address;
                0x0B: Address, Literal;
            },
            SUB => instructions!{
                // M[a] = M[a] - b; no underflow support
                // opcode | a | b:
                0x0C: Address, Address;
                0x0D: Address, Literal;
            },
            JMP => instructions!{
                // Start executing at index of value M[a] or the Literal a-value
                // opcode | a:
                0x0E: Address;
                0x0F: Literal;
            },
            JZ => instructions!{
                // Start executing instructions at index x if M[a] == 0
                // opcode | x | a:
                0x10: Address, Address;
                0x11: Address, Literal;
                0x12: Literal, Address;
                0x13: Literal, Literal;
            },
            JEQ => instructions!{
                // Jump to x or M[x] if M[a] is equal to M[b]
                // or if M[a] is equal to the Literal b.
                // opcode | x | a | b:
                0x14: Address, Address, Address;
                0x15: Literal, Address, Address;
                0x16: Address, Address, Literal;
                0x17: Literal, Address, Literal;
            },
            JLS => instructions!{
                // Jump to x or M[x] if M[a] is less than M[b]
                // or if M[a] is less than the Literal b.
                // opcode | x | a | b:
                // opcode | x | a | b:
                0x18: Address, Address, Address;
                0x19: Literal, Address, Address;
                0x1A: Address, Address, Literal;
                0x1B: Literal, Address, Literal;
            },
            JGT => instructions!{
                // Jump to x or M[x] if M[a] is greater than M[b]
                // or if M[a] is greater than the Literal b
                // opcode | x | a | b:
                0x1C: Address, Address, Address;
                0x1D: Literal, Address, Address;
                0x1E: Address, Address, Literal;
                0x1F: Literal, Address, Literal;
            },
            HALT => instructions!{
                // Halts the program / freeze flow of execution
                0xFF;  // No args
            },
            APRINT => instructions!{
                // Print the contents of M[a] in ASCII
                // opcode | a:
                0x20: Address;
                0x21: Literal;
            },
            DPRINT => instructions!{
                // Print the contents of M[a] as decimal
                // opcode | a:
                0x22: Address;
                0x23: Literal;
            },
            AREAD => instructions!{
                // Custom opcode:
                // Read one char from stdin and store the ASCII value at M[a]
                // opcode | a
                0x24: Address;
            }
        }
    };
}
