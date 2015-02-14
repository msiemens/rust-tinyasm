use rand::distributions::Sample;
use rand::distributions::Range as RandRange;
use rand;

use self::Argument::*;

pub use self::StateChange::*;


// An instruction
pub struct Instruction {
    pub argc: usize,
    arg_types: &'static [Argument],
    implementation: fn(&[u8], &[u8]) -> StateChange
}

impl Instruction {
    pub fn execute(&self, args: &[u8], mem: &[u8]) -> StateChange {
        let decoded_args = decode_args(args, &self.arg_types, mem);
        debug!("interpreted args: {:?}", decoded_args);

        (self.implementation)(&decoded_args, mem)
    }
}


#[derive(Debug)]
pub enum Argument {
    Value,      // The value of an address
    Address,    // An address
    Literal,    // A literal value
}


pub enum StateChange {
    Memset{ address: u8, value: u8 },
    Jump{ address: u8 },
    Halt,
    Continue
}


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


// --- Instruction implementations ----------------------------------------------

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


// Memory Access
make_instruction!(IMov(args[2], memory) {
    Memset { address: args[0], value: args[1] }
});


// Logic operations
make_instruction!(IAnd(args[2], memory) {
    Memset { address: args[0], value: memory[args[0] as usize] & args[1] }
});

make_instruction!(IOr(args[2], memory) {
    Memset { address: args[0], value: memory[args[0] as usize] | args[1] }
});

make_instruction!(IXor(args[2], memory) {
    Memset { address: args[0], value: memory[args[0] as usize] ^ args[1] }
});

make_instruction!(INot(args[1], memory) {
    Memset { address: args[0], value: !memory[args[0] as usize] }
});


// Math
make_instruction!(IAdd(args[2], memory) {
    Memset { address: args[0], value: memory[args[0] as usize] + args[1] }
});

make_instruction!(ISub(args[2], memory) {
    Memset { address: args[0], value: memory[args[0] as usize] - args[1] }
});


// Control
make_instruction!(IHalt -> Halt);

make_instruction!(IJmp(args[1], memory) {
    Jump{ address: args[0] }
});

make_instruction!(IJz(args[2], memory) {
    match args[0] {
        0 => Jump{ address: args[0] },
        _ => Continue
    }
});

make_instruction!(IJeq(args[3], memory) {
    match args[1] == args[2] {
        true => Jump{ address: args[0] },
        false => Continue
    }
});

make_instruction!(IJls(args[3], memory) {
    match args[1] < args[2] {
        true => Jump{ address: args[0] },
        false => Continue
    }
});

make_instruction!(IJgt(args[3], memory) {
    match args[1] > args[2] {
        true => Jump{ address: args[0] },
        false => Continue
    }
});


// I/O
make_instruction!(IAPrint(args[1], memory) -> Continue {
    print!("{:}", args[0] as char);
});

make_instruction!(IDPrint(args[1], memory) -> Continue {
    print!("{:}", args[0]);
});


// Misc
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

macro_rules! instruction(
    ( $instr:ident ) => (
        Instruction {
            argc: 0,
            arg_types: &[],
            implementation: $instr::execute
        }
    );

    ( $instr:ident [ $($t:ident),* ] ) => (
        Instruction {
            arg_types: &[$($t),*],
            argc: count_args!($($t),*),
            implementation: $instr::execute
        }
    );
);

static INSTRUCTIONS_TABLE: &'static [Instruction] = &[
    // M[a] = M[a] bit-wise and M[b]
    // opcode | a | b
    /* 0x00: */ instruction!(IAnd[Address, Value  ]),
    /* 0x01: */ instruction!(IAnd[Address, Literal]),

    // M[a] = M[a] bit-wise or M[b]
    // opcode | a | b
    /* 0x02: */ instruction!(IOr[Address, Value  ]),
    /* 0x03: */ instruction!(IOr[Address, Literal]),

    // M[a] = M[a] bitwise xor M[b]
    // opcode | a | b
    /* 0x04: */ instruction!(IXor[Address, Value  ]),
    /* 0x05: */ instruction!(IXor[Address, Literal]),

    // M[a] = bit-wise not M[a]
    // opcode | a
    /* 0x06: */ instruction!(INot[Address]),

    // M[a] = M[b], or the Literal-set M[a] = b
    // opcode | a | b:
    /* 0x07: */ instruction!(IMov[Address, Value  ]),
    /* 0x08: */ instruction!(IMov[Address, Literal]),

    // M[a] = random value (0 to 25 -> equal probability distribution)
    // opcode | a:
    /* 0x09: */ instruction!(IRandom[Address]),

    // M[a] = M[a] + b | no overflow support
    // opcode | a | b:
    /* 0x0A: */ instruction!(IAdd[Address, Value  ]),
    /* 0x0B: */ instruction!(IAdd[Address, Literal]),

    // M[a] = M[a] - b | no underflow support
    // opcode | a | b:
    /* 0x0C: */ instruction!(ISub[Address, Value]  ),
    /* 0x0D: */ instruction!(ISub[Address, Literal]),

    // Start executing at index of value M[a] or the Literal a-value
    // opcode | a:
    /* 0x0E: */ instruction!(IJmp[Value  ]),
    /* 0x0F: */ instruction!(IJmp[Literal]),

    // Start executing instructions at index x if M[a] == 0
    // opcode | x | a:
    /* 0x10: */ instruction!(IJz[Value,   Value  ]),
    /* 0x11: */ instruction!(IJz[Value,   Literal]),
    /* 0x12: */ instruction!(IJz[Literal, Value  ]),
    /* 0x13: */ instruction!(IJz[Literal, Literal]),

    // Jump to x or M[x] if M[a] is equal to M[b]
    // or if M[a] is equal to the Literal b.
    // opcode | x | a | b:
    /* 0x14: */ instruction!(IJeq[Value,   Value, Value  ]),
    /* 0x15: */ instruction!(IJeq[Literal, Value, Value  ]),
    /* 0x16: */ instruction!(IJeq[Value,   Value, Literal]),
    /* 0x17: */ instruction!(IJeq[Literal, Value, Literal]),

    // Jump to x or M[x] if M[a] is less than M[b]
    // or if M[a] is less than the Literal b.
    // opcode | x | a | b:
    // opcode | x | a | b:
    /* 0x18: */ instruction!(IJls[Value,   Value, Value  ]),
    /* 0x19: */ instruction!(IJls[Literal, Value, Value  ]),
    /* 0x1A: */ instruction!(IJls[Value,   Value, Literal]),
    /* 0x1B: */ instruction!(IJls[Literal, Value, Literal]),

    // Jump to x or M[x] if M[a] is greater than M[b]
    // or if M[a] is greater than the Literal b
    // opcode | x | a | b:
    /* 0x1C: */ instruction!(IJgt[Value,   Value, Value  ]),
    /* 0x1D: */ instruction!(IJgt[Literal, Value, Value  ]),
    /* 0x1E: */ instruction!(IJgt[Value,   Value, Literal]),
    /* 0x1F: */ instruction!(IJgt[Literal, Value, Literal]),

    // Print the contents of M[a] in ASCII
    // opcode | a:
    /* 0x20: */ instruction!(IAPrint[Value  ]),
    /* 0x21: */ instruction!(IAPrint[Literal]),

    // Print the contents of M[a] as decimal
    // opcode | a:
    /* 0x22: */ instruction!(IDPrint[Value  ]),
    /* 0x23: */ instruction!(IDPrint[Literal])
];

// Halts the program / freeze flow of execution
// 0xFF:
static INSTRUCTION_HALT: Instruction = instruction!(IHalt);

pub fn decode_opcode(opcode: u8) -> &'static Instruction {
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