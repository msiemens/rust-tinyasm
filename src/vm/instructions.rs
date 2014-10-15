#[phase(plugin, link)] extern crate log;

extern crate docopt;
#[phase(plugin)]       extern crate seq_macros;
//use std::ascii::StrAsciiExt;
use std::collections::HashMap;
//use std::from_str::FromStr;

//log_syntax!()
//trace_macros!(true)

macro_rules! instruction(
    ( $i:expr ) => (
        box $i as Box<Instruction>
    )
)

macro_rules! make_instruction(
    ( $name:ident [ $argc:expr ]() $body:block ) => (
        make_instruction!($name[$argc](args, memory) $body)
    );

    ( $name:ident [ $argc:expr ] ($args:ident, $mem:ident) $body:block ) => (
        #[deriving(Show)]
        pub struct $name;

        impl Instruction for $name {
            fn execute(&self, $args: &[u8], $mem: &[u8]) -> StateChange {
                $body
            }

            fn argc(&self) -> uint {
                $argc
            }
        }
    );
)


macro_rules! instruction_new(
    ( $i:ident ) => (
        box $i as Box<Instruction>
    );

    ( $i:ident [ $($t:ident),* ] ) => (
        box $i::with_args([$($t),*]) as Box<Instruction>
    );
)

macro_rules! make_instruction_new(
    ( $name:ident [ $argc:expr ]() $body:block ) => (
        pub struct $name;

        impl Instruction for $name {
            #[allow(unused_variables)]
            fn execute(&self, args: &[u8], mem: &[u8]) -> StateChange {
                $body
            }

            fn argc(&self) -> uint {
                0
            }
        }

    );

    ( $name:ident [ $argc:expr ] ($args:ident, $mem:ident) $body:block ) => (
        pub struct $name {
            arg_types: [Argument, ..$argc]
        }

        impl $name {
            fn with_args(types: [Argument, ..$argc]) -> $name {
                $name {
                    arg_types: types
                }
            }

            fn get_args(&self, args: &[u8], mem: &[u8]) -> Vec<u8> {
                self.arg_types.iter().zip(args.iter())
                    .map(|(&ty, &val)| {
                        match ty {
                            Address => mem[val as uint],
                            Literal => val
                        }
                    }).collect()
            }
        }

        impl Instruction for $name {
            fn execute(&self, $args: &[u8], $mem: &[u8]) -> StateChange {
                let $args = self.get_args($args, $mem);
                $body
            }

            fn argc(&self) -> uint {
                $argc
            }
        }
    );
)

pub trait Instruction {
    fn execute(&self, &[u8], &[u8]) -> StateChange;
    fn argc(&self) -> uint;
}

#[deriving(Show)]
pub enum Argument {
    Address,
    Literal
}

pub enum StateChange {
    Memory( /* location: */ u8, /* content: */ u8 ),
    Jump( /* address: */ u8 ),
    Halt,
    Void
}


make_instruction_new!(INop[0]() { // TODO: Only INop {
    Void
})

make_instruction_new!(IHalt[0]() {
    Halt
})


// Memory Access
make_instruction_new!(IMov[2](args, memory) {
    Memory(args[0], args[1])
})
/*make_instruction!(IMov_AA[2](args, memory) {
    Memory(args[0], memory[args[1] as uint])
})

make_instruction!(IMov_AL[2](args, memory) {
    Memory(args[0], args[1])
})*/

// Binary operations
make_instruction_new!(IAnd[2](args, memory) {
    Memory(args[0], args[0] & args[1])
})

/*make_instruction!(IAnd_AA[2](args, memory) {
    Memory(args[0], memory[args[0] as uint] & memory[args[1] as uint])
})

make_instruction!(IAnd_AL[2](args, memory) {
    Memory(args[0], memory[args[0] as uint] & args[1])
})*/


// Arithmetics
make_instruction_new!(IAdd[2](args, memory) {
    Memory(args[0], args[0] + args[1])
})
/*
make_instruction!(IAdd_AA[2](args, memory) {
    Memory(args[0], memory[args[0] as uint] + memory[args[1] as uint])
})

make_instruction!(IAdd_AL[2](args, memory) {
    Memory(args[0], memory[args[0] as uint] + args[1])
})
*/

// I/O
make_instruction_new!(IAPrint[1](args, memory) {
    print!("{:c}", args[0] as char);
    Void
})

make_instruction_new!(IDPrint[1](args, memory) {
    print!("{:u}", args[0]);
    Void
})
/*make_instruction!(IAPrint_A[1](args, memory) {
    print!("{:c}", memory[args[0] as uint] as char);
    Void
})

make_instruction!(IAPrint_L[1](args, memory) {
    print!("{:c}", args[0] as char);
    Void
})

make_instruction!(IDPrint_A[1](args, memory) {
    print!("{:u}", memory[args[0] as uint]);
    Void
})

make_instruction!(IDPrint_L[1](args, memory) {
    print!("{:u}", args[0]);
    Void
})*/

//make_instruction1!(IDPrint[1](args, memory) {
//    print!("{:u}", args0);
//    Void
//})


pub fn get<'a>() -> HashMap<u8, Box<Instruction + 'a>> {
    seq!{
        // M[a] = M[a] bit-wise and M[b]
        // opcode | a | b
        0x00 => instruction_new!(IAnd[Literal, Address]), // TODO: instruction!(IAnd[Address, Address])
        0x01 => instruction_new!(IAnd[Literal, Literal]), //       instruction!(IAnd[Address, Literal])
                                                            //       --> IAnd::with_args(Address, Literal)
                                                            //       --> interpert_args(&self, args)
        // M[a] = M[a] bit-wise or M[b]
        // opcode | a | b
        0x02 => instruction_new!(INop),
        0x03 => instruction_new!(INop),

        // M[a] = M[a] bitwise xor M[b]
        // opcode | a | b
        0x04 => instruction_new!(INop),
        0x05 => instruction_new!(INop),

        // M[a] = bit-wise not M[a]
        // opcode | a
        0x06 => instruction_new!(INop),

        // M[a] = M[b], or the Literal-set M[a] = b
        // opcode | a | b:
        0x07 => instruction_new!(IMov[Literal, Address]),
        0x08 => instruction_new!(IMov[Literal, Literal]),

        // M[a] = random value (0 to 25 -> equal probability distribution)
        // opcode | a:
        0x09 => instruction_new!(INop),

        // M[a] = M[a] + b | no overflow support
        // opcode | a | b:
        0x0A => instruction_new!(IAdd[Address, Address]),
        0x0B => instruction_new!(IAdd[Address, Literal]),

        // M[a] = M[a] - b | no underflow support
        // opcode | a | b:
        0x0C => instruction_new!(INop),
        0x0D => instruction_new!(INop),

        // Start executing at index of value M[a] or the Literal a-value
        // opcode | a:
        0x0E => instruction_new!(INop),
        0x0F => instruction_new!(INop),

        // Start executing instructions at index x if M[a] == 0
        // opcode | x | a:
        0x10 => instruction_new!(INop),
        0x11 => instruction_new!(INop),
        0x12 => instruction_new!(INop),
        0x13 => instruction_new!(INop),

        // Jump to x or M[x] if M[a] is equal to M[b]
        // or if M[a] is equal to the Literal b.
        // opcode | x | a | b:
        0x14 => instruction_new!(INop),
        0x15 => instruction_new!(INop),
        0x16 => instruction_new!(INop),
        0x17 => instruction_new!(INop),

        // Jump to x or M[x] if M[a] is less than M[b]
        // or if M[a] is less than the Literal b.
        // opcode | x | a | b:
        // opcode | x | a | b:
        0x18 => instruction_new!(INop),
        0x19 => instruction_new!(INop),
        0x1A => instruction_new!(INop),
        0x1B => instruction_new!(INop),

        // Jump to x or M[x] if M[a] is greater than M[b]
        // or if M[a] is greater than the Literal b
        // opcode | x | a | b:
        0x1C => instruction_new!(INop),
        0x1D => instruction_new!(INop),
        0x1E => instruction_new!(INop),
        0x1F => instruction_new!(INop),

        // Halts the program / freeze flow of execution
        0xFF => instruction_new!(IHalt),

        // Print the contents of M[a] in ASCII
        // opcode | a:
        0x20 => instruction_new!(IAPrint[Address]),
        0x21 => instruction_new!(IAPrint[Literal]),

        // Print the contents of M[a] as decimal
        // opcode | a:
        0x22 => instruction_new!(IDPrint[Address]),
        0x23 => instruction_new!(IDPrint[Literal]),

        // Custom opcode:
        // Read one char from stdin and store the ASCII value at M[a]
        // opcode | a
        0x24 => instruction_new!(INop)
    }
}

/*fn opcode_into_str(opcode: u8) -> String {
    match {0x00 | 0x01 => ""}.into_string()
}*/