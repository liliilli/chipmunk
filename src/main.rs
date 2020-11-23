use std::env;
use std::time;
use std::io::{Read};
use std::fs;
use std::mem;

mod engine;
use engine::isa::{parse_instruction, to_bitfield_string};
use engine::register::{Registers};
use engine::memory::{Memory};
use engine::screen::{Screen, DrawMessage, PixelState};

extern crate pancurses;
use pancurses::{initscr, endwin, Input, noecho, resize_term};

static FULL_BLOCK_CHAR: u16 = 0x2588;

fn is_file_valid_ch8(path: &str) -> bool {
    use std::path::Path;

    let path = Path::new(&path);
    if !(path.exists() && path.is_file()) {
        // If file is not exist, and not file, just return false.
        return false;
    } 

    // Check file length is a multiply of 2 bytes.
    // Every Chip-8 instruction has 2 bytes.
    match fs::metadata(path) {
        Ok(metadata) if (metadata.len() as usize % mem::size_of::<u16>()) == 0 => (),
        _ => return false,
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

fn get_ch8_file_path(args: &mut env::Args) -> Result<String, String> {
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

fn main() {
    // Get file path.
    // Interpret file and check validation.
    let mut args = env::args();
    let file_path = match get_ch8_file_path(&mut args) {
        Ok(path) => path,
        Err(err_msg) => {
            println!("{}", err_msg);
            return;
        }
    };

    // Construct memory (4KiB)
    // Verbose? Check memory state.
    let memory = Memory::new(&file_path).unwrap();
    //memory.print_memory_dump();

    // Set registers
    let mut registers = Registers::new();
    println!("Registers\n{}", registers);

    // Set screen buffer
    let mut screen = Screen::new();
    let block_str = String::from_utf16(&[FULL_BLOCK_CHAR]).unwrap();

    // Set ncurse window (GUI)
    let window = initscr();
    resize_term(32, 64);
    window.keypad(true);
    window.refresh();
    window.nodelay(true);
    //window.printw(String::from_utf16(&[HALF_BLOCK_CHAR, FULL_BLOCK_CHAR]).unwrap());
    noecho();

    loop {
        // Input
        match window.getch() {
            //Some(Input::Character(_)) => (),
            //Some(input) => { window.addstr(&format!("{:?}", input)); },
            Some(Input::KeyDC) => break,
            _ => (),
        }

        // Update 
        let now_pc = registers.get_pc();
        if let Some(inst) = memory.parse_instruction(now_pc) {
            use engine::register::SideEffect;
            // Execute instruction
            // Update register (just updating) and consequential side effects.
            match registers.update_registers(inst) {
            Some(SideEffect::ClearDisplay) => {
                screen.clear();
                window.clear();
            },
            Some(SideEffect::Draw{ pos, n, l: addr }) => {
                let bytes = memory.get_data_bytes(addr as usize, n as usize);
                let dirty_pixels = screen.draw(pos, &bytes);
                for DrawMessage { pos: (x, y), state } in &dirty_pixels {
                    window.mv(*y as i32, *x as i32);
                    match state {
                        PixelState::Erased => { window.delch(); () },
                        PixelState::Drawn => { window.printw(&block_str); () },
                    }
                }
            }   
            _ => (),
            }
            //println!("{}", registers);
        } else { 
            // Failure. Abort program.
            break;
        }
    }

    endwin();
}
