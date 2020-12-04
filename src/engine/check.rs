use crate::engine::isa::{to_bitfield_string, parse_instruction};
use std::{
    fs, env,
    io::Read
};

fn is_file_valid_ch8(path: &str) -> bool {
    use std::path::Path;

    let path = Path::new(&path);
    if !(path.exists() && path.is_file()) {
        // If file is not exist, and not file, just return false.
        return false;
    } 

    // Read file and check validation.
    let file = {
        if let Ok(file) = fs::File::open(path) {
            file
        } else {
            return false;
        }
    };

    enum InstructionState { Left, Right, }
    let mut instruction: [u8; 2] = [0, 0];
    let mut parse_state = InstructionState::Left;
    let mut address = 0x200;
    for byte in file.bytes() {
        // Error check
        if let Err(_) = byte { return false; }

        // Parse
        let byte = byte.unwrap();
        let (next_state, check_instruction) = match parse_state {
            InstructionState::Left => { instruction[0] = byte; (InstructionState::Right, false) },
            InstructionState::Right => { instruction[1] = byte; (InstructionState::Left, true) }
        };

        // Check instruction
        if check_instruction {
            println!(
                "{:04} : 0x{:02x}{:02x} :: {} :: {:?}", 
                address, instruction[0], instruction[1], 
                to_bitfield_string(&instruction, '1', '0'),
                parse_instruction(&instruction)
            );
            address += 0x02; // 2 Bytes
        }

        // Update flag.
        parse_state = next_state;
    }

    true
}

pub fn get_ch8_file_path(args: &mut env::Args) -> Result<String, String> {
    // Check given arguments are valid.
    if args.len() != 2 {
        return Err(format!("Valid usage : ./{} {}", "sh_chip8.exe", "valid ch8 file path"));
    } 

    // Check file is exist, and valid.
    let file_path: String = args.nth(1).unwrap();
    if is_file_valid_ch8(&file_path) == true {
        Ok(file_path)
    } else {
        return Err(format!("Valid usage : ./{} {}", "sh_chip8.exe", "valid ch8 file path"))
    }
}

