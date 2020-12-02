use std::fs;
use std::io::Read;
use super::isa;

pub struct Memory {
    memory: Vec<u8>,
}

impl Memory {
    pub fn new(valid_file_path: &str) -> Option<Memory> {
        let mut memory = Vec::<u8>::new();
        memory.resize(4 << 10, 0);

        // Set initial memory.


        // Set file memory.
        // Read file.
        let mut file = {
            if let Ok(file) = fs::File::open(valid_file_path) {
                file
            } else {
                println!("Unexpected error occurred.");
                return None;
            }
        };

        // Copy data (instruction & data) into vec.
        let mut data_buffer = Vec::<u8>::new();
        match file.read_to_end(&mut data_buffer) {
            Ok(_) => (),
            Err(_) => return None,
        }

        // Copy to 0x512~ of memory (to 4KiB)
        for (t, r) in memory.iter_mut().skip(0x200).zip(data_buffer.iter_mut()) {
            *t = *r;
        }

        // Return
        Some(Memory { memory })
    }

    ///
    pub fn print_memory_dump(&self) {
        enum InstructionState { Left, Right, }
        let mut instruction: [u8; 2] = [0, 0];
        let mut parse_state = InstructionState::Left;
        let mut address = 0usize;

        for byte in &self.memory {
            // Parse [u8, 2]
            let (next_state, check_instruction) = match parse_state {
            InstructionState::Left  => { instruction[0] = *byte; (InstructionState::Right, false) },
            InstructionState::Right => { instruction[1] = *byte; (InstructionState::Left, true) }
            };

            // Check instruction
            if check_instruction {
                if address % 0x20 == 0 { print!("\n{:04} : ", address); }

                print!("{:02x}{:02x} ", instruction[0], instruction[1]);
                address += 0x02; // 2 Bytes
            }

            // Update flag.
            // Update address variable and line-feed when the condition is satisfied.
            parse_state = next_state;
        }
        println!();
    }

    pub fn parse_instruction(&self, addr: u16) -> Option<isa::Instruction> {
        // Check out of range exception.
        if (addr + 1) as usize >= self.memory.len() { return None; } 

        // Parse instruction.
        let addr = addr as usize;
        let bytes: [u8; 2] = [self.memory[addr], self.memory[addr + 1]];

        isa::parse_instruction(&bytes)
    }

    pub fn get_data_bytes(&self, addr: usize, count: usize) -> Vec<u8> {
        assert!(addr < (4usize << 10));

        self.memory.iter()
            .skip(addr)
            .take(count)
            .map(|&x| x)
            .collect()
    }

    pub fn store_from(&mut self, dump_vals: &[u8], mut l: u16 ) {
        for &val in dump_vals {
            assert!(l < (4 << 10));
            self.memory[l as usize] = val;
            l += 1;
        }
    }
}

