use std::io::File;

use super::Args;
use self::instructions::{INSTRUCTIONS, ModifyMemory, Jump, Halt, Continue};


mod instructions;


pub fn main(args: Args) {
    let source = match File::open(&Path::new(args.arg_input)).read_to_end() {
        Ok(v)  => v,
        Err(e) => { println!("Can't read file: {}", e); return; }
    };

    run(&*source);
}

fn run(source: &[u8]) {
    let mut memory = [0u8; 256];
    let mut pc = 0us;

    loop {
        debug!("");
        debug!("source: {:?}@{}", source, source.len());
        debug!("memory: {:?}@{}", &memory[], memory.len());
        debug!("pc: {}", pc);

        let opcode = source[pc];                        debug!("opcode: {:#04X}", opcode);
        let ref instruction = INSTRUCTIONS.get(&opcode).unwrap();
        let argc = instruction.argc();                  debug!("argc: {}", argc);
        if pc + 1 + argc >= source.len() {
            panic!("Reached end of input without HALT!")
        }
        let argv = match argc {
            0 => &[][],  // Empty slice
            _ => &source[pc + 1 .. pc + 1 + argc][]
        };                                              debug!("argv: {:?}", argv);
        pc += 1 + argc;  // Increment programm counter


        match instruction.execute(argv, &memory) {
            Halt => break,
            Jump(address) => {
                debug!("Jumping to {}", address);
                pc = address as usize;
            },
            ModifyMemory(location, content) => {
                debug!("Setting m[{}] = {}", location, content);
                memory[location as usize] = content;
            },
            Continue => {
            }
        }
    }
}