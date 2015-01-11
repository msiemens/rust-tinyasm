use std::collections::HashMap;
use std::rand::distributions::Sample;
use std::rand::distributions::Range as RandRange;
use std::rand;

use self::Argument::*;

pub use self::StateChange::*;


pub trait Instruction {
    fn execute(&self, &[u8], &[u8]) -> StateChange;
    fn argc(&self) -> usize;
}

#[derive(Show)]
pub enum Argument {
    Address,
    Literal
}

pub enum StateChange {
    ModifyMemory( /* location: */ u8, /* content: */ u8 ),
    Jump( /* address: */ u8 ),
    Halt,
    Continue
}


macro_rules! fn_execute(
    ($args:ident/unused, $mem:ident, $body:block) => {
        fn execute(&self, $args: &[u8], $mem: &[u8]) -> StateChange {
            let $args = self.decode_args($args, $mem);
            $body
        }
    };

    ($args:ident / $raw:ident, $mem:ident, $body:block) => {
        fn execute(&self, $args: &[u8], $mem: &[u8]) -> StateChange {
            let $raw = $args;
            let $args = self.decode_args($args, $mem);
            debug!("interpreted args: {:?}", $args);
            $body
        }
    };
);

macro_rules! make_instruction(
    // Actual implementation
    ( $name:ident ($args:ident / $raw:ident [ $argc:expr ] , $mem:ident) $body:block ) => {
        pub struct $name {
            arg_types: [Argument; $argc]
        }

        impl $name {
            #[inline]
            fn decode_args(&self, args: &[u8], mem: &[u8]) -> Vec<u8> {
                self.arg_types.iter()
                    .zip(args.iter())
                    .map(|(ty, val)| {
                        match *ty {
                            Argument::Address => mem[*val as usize],
                            Argument::Literal => *val
                        }
                    }).collect()
            }
        }

        impl Instruction for $name {
            fn_execute!($args/$raw, $mem, $body);

            fn argc(&self) -> usize {
                $argc
            }
        }
    };


    // Arguments and return type with raw argument access
    ( $name:ident ($args:ident [ $argc:expr ] , $mem:ident) -> $ret_type:ident ( $raw:ident [ $mem_arg:expr ] ) $body:block ) => {
        make_instruction!($name($args/$raw[$argc], $mem) {
            let result = $body;
            $ret_type($raw[$mem_arg], result)
        });
    };

    // Arguments and simple return type
    ( $name:ident ($args:ident [ $argc:expr ] , $mem:ident) -> $ret_type:ident $body:block ) => {
        make_instruction!($name($args[$argc], $mem) {
            $body;
            $ret_type
        });
    };

    // Simple arguments
    ( $name:ident ($args:ident [ $argc:expr ] , $mem:ident) $body:block ) => {
        make_instruction!($name($args/unused[$argc], $mem) $body);
    };

    // No arguments
    ( $name:ident $body:block ) => {
        pub struct $name;

        impl Instruction for $name {
            #[allow(unused_variables)]
            fn execute(&self, args: &[u8], mem: &[u8]) -> StateChange {
                $body
            }

            fn argc(&self) -> usize {
                0
            }
        }
    };
);


make_instruction!(IHalt { Halt });


// Memory Access
make_instruction!(IMov(args[2], memory) -> ModifyMemory(raw[0]) {
    args[1]
});


// Logic operations
make_instruction!(IAnd(args[2], memory) -> ModifyMemory(raw[0]) {
    args[0] & args[1]
});

make_instruction!(IOr(args[2], memory) -> ModifyMemory(raw[0]) {
    args[0] | args[1]
});

make_instruction!(IXor(args[2], memory) -> ModifyMemory(raw[0]) {
    args[0] ^ args[1]
});

make_instruction!(INot(args[1], memory) -> ModifyMemory(raw[0]) {
    !args[0]
});


// Math
make_instruction!(IAdd(args[2], memory) -> ModifyMemory(raw[0]) {
    args[0] + args[1]
});

make_instruction!(ISub(args[2], memory) -> ModifyMemory(raw[0]) {
    args[0] - args[1]
});


// Control
make_instruction!(IJmp(args[1], memory) {
    Jump(args[0])
});

make_instruction!(IJz(args[2], memory) {
    match args[0] {
        0 => Jump(args[0]),
        _ => Continue
    }
});

make_instruction!(IJeq(args[3], memory) {
    match args[1] == args[2] {
        true => Jump(args[0]),
        false => Continue
    }
});

make_instruction!(IJls(args[3], memory) {
    match args[1] < args[2] {
        true => Jump(args[0]),
        false => Continue
    }
});

make_instruction!(IJgt(args[3], memory) {
    match args[1] > args[2] {
        true => Jump(args[0]),
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
make_instruction!(IRandom(args[1], memory) -> ModifyMemory(raw[0]) {
    let mut rand_range = RandRange::new(0u8, 255u8);
    let mut rng = rand::thread_rng();
    rand_range.sample(&mut rng)
});


macro_rules! instruction(
    ( $i:ident ) => (
        box $i as Box<Instruction>
    );

    ( $i:ident [ $($t:ident),* ] ) => (
        box $i { arg_types: [$($t),*] } as Box<Instruction>
    );
);

lazy_static! {
    pub static ref INSTRUCTIONS: HashMap<u8, Box<Instruction + 'static>> = {
        seq!{
            // M[a] = M[a] bit-wise and M[b]
            // opcode | a | b
            0x00 => instruction!(IAnd[Address, Address]),
            0x01 => instruction!(IAnd[Address, Literal]),

            // M[a] = M[a] bit-wise or M[b]
            // opcode | a | b
            0x02 => instruction!(IOr[Address, Address]),
            0x03 => instruction!(IOr[Address, Literal]),

            // M[a] = M[a] bitwise xor M[b]
            // opcode | a | b
            0x04 => instruction!(IXor[Address, Address]),
            0x05 => instruction!(IXor[Address, Literal]),

            // M[a] = bit-wise not M[a]
            // opcode | a
            0x06 => instruction!(INot[Address]),

            // M[a] = M[b], or the Literal-set M[a] = b
            // opcode | a | b:
            0x07 => instruction!(IMov[Address, Address]),
            0x08 => instruction!(IMov[Address, Literal]),

            // M[a] = random value (0 to 25 -> equal probability distribution)
            // opcode | a:
            0x09 => instruction!(IRandom[Address]),

            // M[a] = M[a] + b | no overflow support
            // opcode | a | b:
            0x0A => instruction!(IAdd[Address, Address]),
            0x0B => instruction!(IAdd[Address, Literal]),

            // M[a] = M[a] - b | no underflow support
            // opcode | a | b:
            0x0C => instruction!(ISub[Address, Address]),
            0x0D => instruction!(ISub[Address, Literal]),

            // Start executing at index of value M[a] or the Literal a-value
            // opcode | a:
            0x0E => instruction!(IJmp[Address]),
            0x0F => instruction!(IJmp[Literal]),

            // Start executing instructions at index x if M[a] == 0
            // opcode | x | a:
            0x10 => instruction!(IJz[Address, Address]),
            0x11 => instruction!(IJz[Address, Literal]),
            0x12 => instruction!(IJz[Literal, Address]),
            0x13 => instruction!(IJz[Literal, Literal]),

            // Jump to x or M[x] if M[a] is equal to M[b]
            // or if M[a] is equal to the Literal b.
            // opcode | x | a | b:
            0x14 => instruction!(IJeq[Address, Address, Address]),
            0x15 => instruction!(IJeq[Literal, Address, Address]),
            0x16 => instruction!(IJeq[Address, Address, Literal]),
            0x17 => instruction!(IJeq[Literal, Address, Literal]),

            // Jump to x or M[x] if M[a] is less than M[b]
            // or if M[a] is less than the Literal b.
            // opcode | x | a | b:
            // opcode | x | a | b:
            0x18 => instruction!(IJls[Address, Address, Address]),
            0x19 => instruction!(IJls[Literal, Address, Address]),
            0x1A => instruction!(IJls[Address, Address, Literal]),
            0x1B => instruction!(IJls[Literal, Address, Literal]),

            // Jump to x or M[x] if M[a] is greater than M[b]
            // or if M[a] is greater than the Literal b
            // opcode | x | a | b:
            0x1C => instruction!(IJgt[Address, Address, Address]),
            0x1D => instruction!(IJgt[Literal, Address, Address]),
            0x1E => instruction!(IJgt[Address, Address, Literal]),
            0x1F => instruction!(IJgt[Literal, Address, Literal]),

            // Halts the program / freeze flow of execution
            0xFF => instruction!(IHalt),

            // Print the contents of M[a] in ASCII
            // opcode | a:
            0x20 => instruction!(IAPrint[Address]),
            0x21 => instruction!(IAPrint[Literal]),

            // Print the contents of M[a] as decimal
            // opcode | a:
            0x22 => instruction!(IDPrint[Address]),
            0x23 => instruction!(IDPrint[Literal]),

            // Custom opcode:
            // Read one char from stdin and store the ASCII value at M[a]
            // opcode | a
            //0x24 => instruction!(INop)
        }
    };
}