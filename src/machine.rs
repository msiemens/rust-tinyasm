use std::ascii::AsciiExt;
use std::collections::HashMap;
use std::str::FromStr;
use rand::distributions::Sample;
use rand::distributions::Range as RandRange;
use rand;

use self::Argument::*;

pub use self::StateChange::*;


// --- Instruction + helpers ---------------------------------------------

/// Representation of an instruction (opcode + args + implementation)
pub struct Instruction {
    pub opcode: u8,
    pub argc: usize,
    pub arg_types: &'static [Argument],
    implementation: fn(&[u8], &[u8]) -> StateChange
}

impl Instruction {
    pub fn execute(&self, args: &[u8], mem: &[u8]) -> StateChange {
        let decoded_args = decode_args(args, &self.arg_types, mem);
        debug!("interpreted args: {:?}", decoded_args);

        (self.implementation)(&decoded_args, mem)
    }
}


/// Argument types
#[derive(Debug, PartialEq)]
pub enum Argument {
    Value,      // The value of an address
    Address,    // An address
    Literal,    // A literal value
}


/// Possible results of instruction execution
pub enum StateChange {
    Memset { address: u8, value: u8 },
    Jump { address: u8 },
    Halt,
    Continue
}


// --- Instruction helpers ------------------------------------------------------

/// A helper that decodes an instruction's arguments. That means it reads
/// values from memory where addresses are passed.
fn decode_args(args: &[u8], arg_types: &[Argument], mem: &[u8]) -> Vec<u8> {
    arg_types.iter()
        .zip(args.iter())
        .map(|(ty, val)| {
            match *ty {
                Argument::Value => mem[*val as usize],
                Argument::Address => *val,
                Argument::Literal => *val,
            }
        })
        .collect()
}

/// A helper to define an instruction
macro_rules! make_instruction {
    // Static return
    ($name:ident -> $ret_type:ident) => {
        pub struct $name;
        impl $name {
            #[allow(unused_variables)]
            fn execute(args: &[u8], mem: &[u8]) -> StateChange {
                $ret_type
            }
        }
    };

    // Arguments and static return type
    ( $name:ident ($args:ident [ $argc:expr ] , $mem:ident) -> $ret_type:ident $body:block ) => {
        pub struct $name;
        impl $name {
            #[allow(unused_variables)]
            fn execute($args: &[u8], $mem: &[u8]) -> StateChange {
                $body;
                $ret_type
            }
        }
    };

    // Normal arguments
    ( $name:ident ($args:ident [ $argc:expr ] , $mem:ident) $body:block ) => {
        pub struct $name;
        impl $name {
            #[allow(unused_variables)]
            fn execute($args: &[u8], $mem: &[u8]) -> StateChange {
                $body
            }
        }
    };
}


// --- Instruction implementations ----------------------------------------------
// Syntax of the comments:
//    a, b, c: first/second/third argument
//    M[x]: Value of address x

// --- Memory Access

// M[a] = M[b], or the Literal-set M[a] = b
make_instruction!(IMov(args[2], memory) {
    Memset { address: args[0], value: args[1] }
});


// --- Logic operations

// M[a] = M[a] & M[b]
make_instruction!(IAnd(args[2], memory) {
    Memset { address: args[0], value: memory[args[0] as usize] & args[1] }
});

// M[a] = M[a] | M[b]
make_instruction!(IOr(args[2], memory) {
    Memset { address: args[0], value: memory[args[0] as usize] | args[1] }
});

// M[a] = M[a] ^ M[b]
make_instruction!(IXor(args[2], memory) {
    Memset { address: args[0], value: memory[args[0] as usize] ^ args[1] }
});

// M[a] = !M[a]
make_instruction!(INot(args[1], memory) {
    Memset { address: args[0], value: !memory[args[0] as usize] }
});


// --- Math

// M[a] = M[a] + b
make_instruction!(IAdd(args[2], memory) {
    Memset { address: args[0], value: memory[args[0] as usize] + args[1] }
});


// M[a] = M[a] - b
make_instruction!(ISub(args[2], memory) {
    Memset { address: args[0], value: memory[args[0] as usize] - args[1] }
});


// --- Control

make_instruction!(IHalt -> Halt);

// Jump to a
make_instruction!(IJmp(args[1], memory) {
    Jump{ address: args[0] }
});

// Jump to a if b == 0
make_instruction!(IJz(args[2], memory) {
    match args[0] {
        0 => Jump{ address: args[0] },
        _ => Continue
    }
});

// Jump to a if b == c
make_instruction!(IJeq(args[3], memory) {
    match args[1] == args[2] {
        true => Jump{ address: args[0] },
        false => Continue
    }
});

// Jump to a if b < c
make_instruction!(IJls(args[3], memory) {
    match args[1] < args[2] {
        true => Jump{ address: args[0] },
        false => Continue
    }
});

// Jump to a if b > c
make_instruction!(IJgt(args[3], memory) {
    match args[1] > args[2] {
        true => Jump{ address: args[0] },
        false => Continue
    }
});


// --- I/O

// Print the contents of M[a] in ASCII
make_instruction!(IAPrint(args[1], memory) -> Continue {
    print!("{:}", args[0] as char);
});

// Print the contents of M[a] in decimal
make_instruction!(IDPrint(args[1], memory) -> Continue {
    print!("{:}", args[0]);
});


// --- Misc

// M[a] = random value (0 to 25 -> equal probability distribution)
make_instruction!(IRandom(args[1], memory) {
    let mut rand_range = RandRange::new(0u8, 255u8);
    let mut rng = rand::thread_rng();
    Memset { address: args[0], value: rand_range.sample(&mut rng) }
});


// --- Opcode -> Instruction mapping --------------------------------------------

macro_rules! count_args {
    () => { 0 };
    ($x:expr) => { 1 };
    ($head:expr, $($tail:expr),+) => { 1 + count_args!($($tail),+) };
}

macro_rules! instruction {
    ( $opcode:expr => $instr:ident ) => (
        Instruction {
            opcode: $opcode,
            argc: 0,
            arg_types: &[],
            implementation: $instr::execute
        }
    );

    ( $opcode:expr => $instr:ident [ $($t:ident),* ] ) => (
        Instruction {
            opcode: $opcode,
            arg_types: &[$($t),*],
            argc: count_args!($($t),*),
            implementation: $instr::execute
        }
    );
}

macro_rules! instructions {
    ( $($mnem:ident : $( $opcode:expr => $instr:ident [ $($t:ident),* ] ),* ; )* ) => {

        // Remember: HALT is not part of the macro's arguments as its opcode
        // doesn't follow the scheme of the other instructions.

        #[derive(Clone, PartialEq, Eq, Debug, Hash)]
        pub enum Mnemonic {
            $( $mnem, )* HALT
        }

        impl FromStr for Mnemonic {
            type Err = String;

            fn from_str(s: &str) -> Result<Mnemonic, String> {
                match &*s.to_ascii_uppercase() {
                    $(
                        stringify!($mnem) => Ok(Mnemonic::$mnem),
                    )*
                    "HALT" => Ok(Mnemonic::HALT),
                    _ => Err(format!("Invalid instruction: {}", s))
                }
            }
        }

        /// An opcode → instruction mapping
        static INSTRUCTIONS_TABLE: &'static [Instruction] = &[
            $(
                $(
                    instruction!($opcode => $instr [ $($t),* ])
                ),*
            ),*
        ];


        /// An mnemonic → instructions mapping + access methods
        pub struct InstructionManager {
            map: HashMap<Mnemonic, Vec<&'static Instruction>>
        }

        impl InstructionManager {
            pub fn new() -> InstructionManager {
                let mut map = HashMap::new();
                $(
                    map.insert(Mnemonic::$mnem, vec![
                        $( &INSTRUCTIONS_TABLE[$opcode] ),*
                    ]);
                )*

                map.insert(Mnemonic::HALT, vec![&INSTRUCTION_HALT]);

                InstructionManager {
                    map: map
                }
            }

            pub fn lookup_operations(&self, mnem: Mnemonic) -> &[&'static Instruction] {
                &self.map[mnem]
            }

            pub fn decode_opcode(&self, opcode: u8) -> &'static Instruction {
                // We're assuming the table is not full
                assert!(INSTRUCTIONS_TABLE.len() < 0xFF);

                if opcode != 0xFF && opcode as usize >= INSTRUCTIONS_TABLE.len() {
                    panic!("Invalid opcode: {}", opcode)
                };

                // Special case: 0xFF is HALT
                if opcode == 0xFF {
                    &INSTRUCTION_HALT
                } else {
                    &INSTRUCTIONS_TABLE[opcode as usize]
                }
            }

        }
    };
}

instructions! {
    AND:
    0x00 => IAnd[Address, Value  ],
    0x01 => IAnd[Address, Literal];

    OR:
    0x02 => IOr[Address, Value  ],
    0x03 => IOr[Address, Literal];

    XOR:
    0x04 => IXor[Address, Value  ],
    0x05 => IXor[Address, Literal];

    NOT:
    0x06 => INot[Address];


    MOV:
    0x07 => IMov[Address, Value  ],
    0x08 => IMov[Address, Literal];


    RANDOM:
    0x09 => IRandom[Address];

    ADD:
    0x0A => IAdd[Address, Value  ],
    0x0B => IAdd[Address, Literal];

    SUB:
    0x0C => ISub[Address, Value]  ,
    0x0D => ISub[Address, Literal];


    JMP:
    0x0E => IJmp[Value  ],
    0x0F => IJmp[Literal];

    JZ:
    0x10 => IJz[Value,   Value  ],
    0x11 => IJz[Value,   Literal],
    0x12 => IJz[Literal, Value  ],
    0x13 => IJz[Literal, Literal];

    JEQ:
    0x14 => IJeq[Value,   Value, Value  ],
    0x15 => IJeq[Literal, Value, Value  ],
    0x16 => IJeq[Value,   Value, Literal],
    0x17 => IJeq[Literal, Value, Literal];

    JLS:
    0x18 => IJls[Value,   Value, Value  ],
    0x19 => IJls[Literal, Value, Value  ],
    0x1A => IJls[Value,   Value, Literal],
    0x1B => IJls[Literal, Value, Literal];

    JGT:
    0x1C => IJgt[Value,   Value, Value  ],
    0x1D => IJgt[Literal, Value, Value  ],
    0x1E => IJgt[Value,   Value, Literal],
    0x1F => IJgt[Literal, Value, Literal];


    APRINT:
    0x20 => IAPrint[Value  ],
    0x21 => IAPrint[Literal];

    DPRINT:
    0x22 => IDPrint[Value  ],
    0x23 => IDPrint[Literal];
}

// Halt the program
static INSTRUCTION_HALT: Instruction = instruction!(0xFF => IHalt);