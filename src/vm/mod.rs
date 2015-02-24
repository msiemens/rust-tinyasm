use std::fs::File;
use std::io::Read;
use std::path::Path;

use machine::{InstructionManager, Memset, Jump, Halt, Continue};
use Args;


const MEMORY_SIZE: usize = 256;


pub fn main(args: Args) {
    // Read binary file
    let path = Path::new(&args.arg_input);
    let mut file = match File::open(&path) {
        Ok(f) => f,
        Err(err) => { panic!("Can't open {}: {}", path.display(), err) }
    };

    let mut source = vec![];
    match file.read_to_end(&mut source) {
        Ok(v)  => v,
        Err(err) => { panic!("Can't read {}: {}", path.display(), err) }
    };

    // Run virtual machine
    run(&source);
}

fn run(source: &[u8]) {
    let mut memory = [0u8; MEMORY_SIZE];
    let mut ip = 0;
    let im = InstructionManager::new();

    loop {
        debug!("--- next instruction (ip: {})", ip);
        debug!("memory: {:?}@{}", &memory[..], memory.len());

        // Step 1: Read instruction
        let opcode = source[ip];

        // Step 2: Decode opcode and read + decode the arguments
        let ref instruction = im.decode_opcode(opcode);

        let argc = instruction.argc;
        if ip + argc >= source.len() {
            panic!("Reached end of input without HALT!")
        }
        let args = &source[ip + 1 .. ip + 1 + argc];

        let decoded_args = im.decode_args(args, instruction.arg_types, &memory);

        // Step 3 + 4: Execute instruction and process result
        debug!("executing {:?} ({:#04X}) with {:?}", instruction.mnem, opcode, decoded_args);

        match instruction.execute(decoded_args, &memory) {
            Continue => {},
            Jump { address } => {
                debug!("Jumping to {}", address);
                ip = address as usize;
                continue;  // We've already updated the instruction pointer
            },
            Memset { address, value } => {
                debug!("Setting m[{}] = {}", address, value);
                memory[address as usize] = value;
            },
            Halt => break
        }

        // Update instruction pointer
        ip += 1;  // Skip opcode
        ip += argc;  // Skip args
    }
}