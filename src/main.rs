use std::env;
use std::time;
use std::io::{Read};
use std::fs;

mod engine;
use engine::isa::{parse_instruction, to_bitfield_string};
use engine::register::{Registers};
use engine::memory::{Memory};
use engine::screen::{Screen, DrawMessage, PixelState};
use engine::keypad::Keypad;
use engine::state::MachineState;

extern crate pancurses;
use pancurses::{initscr, endwin, Input, noecho, resize_term, beep};

static FULL_BLOCK_CHAR: u16 = 0x2588;

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

    // Set devices of CHIP-8 simulator.
    let mut memory = Memory::new(&file_path).unwrap();
    let mut registers = Registers::new();
    let mut screen = Screen::new();
    let mut keypad = Keypad::new(); // Already reseted.
    let mut machine_state = MachineState::Normal;

    // Set ncurse window (GUI)
    let window = initscr();
    resize_term(32, 64);
    window.keypad(true);
    window.refresh();
    window.nodelay(true);
    noecho();

    // Create block string for expressing pixel drawing into ncurse display.
    let block_str = String::from_utf16(&[FULL_BLOCK_CHAR]).unwrap();

    // Start one frame.
    loop {
        // Input keypad.
        let input_keyval = match window.getch() {
            Some(Input::Character(chr)) => keypad.set_press(chr),
            Some(Input::KeyExit) => break,
            _ => None,
        };

        // If machine state is waiting for key press, and some valueable key is pressed,
        // Change machine state and process side-effect.
        if let MachineState::WaitKeyPress{ r } = machine_state {
            if let Some(keyval) = input_keyval {
                registers.update_general_register(r, keyval);
                machine_state = MachineState::Normal;
            }
        }

        if machine_state == MachineState::Normal {
            // Parse instruction and process.
            if let Some(instruction) = memory.parse_instruction(registers.get_pc()) {
                use engine::register::SideEffect;

                // Update register with instruction.
                let side_effect = registers.update_registers(instruction);

                // Process consequential side effects.
                match side_effect {
                    Some(SideEffect::ClearDisplay) => {
                        screen.clear();
                        window.clear();
                    },
                    Some(SideEffect::Draw{ pos, n, l: addr }) => {
                        // Update screen buffer and get dirty pixels to update window buffer.
                        // New carry flag value will be returned.
                        let (dirty_pixels, is_any_erased) = screen.draw(
                            pos, 
                            &memory.get_data_bytes(addr as usize, n as usize)
                        );

                        // Update VF (carry & borrow flag)
                        registers.update_vf(is_any_erased);

                        // Update window buffer.
                        for DrawMessage { pos: (x, y), state } in &dirty_pixels {
                            window.mv(*y as i32, *x as i32);
                            match state {
                                PixelState::Erased => { window.delch(); () },
                                PixelState::Drawn => { window.printw(&block_str); () },
                            }
                        }
                    },
                    Some(SideEffect::MemDump{ dump_vals, l }) => {
                        // 
                        memory.store_from(&dump_vals, l);
                    },
                    Some(SideEffect::MemRead{ count, l }) => {
                        // First, get values from memory [l, l + count)
                        // Second, store from v0 to v0 + (count - 1).
                        registers.store_from_v0(&memory.get_data_bytes(l as usize, count as usize));
                    },
                    Some(SideEffect::WaitKeyPress{ r }) => {
                        machine_state = MachineState::WaitKeyPress{ r };
                    },
                    None => (),
                }
            } else { 
                // Failure. Abort program.
                println!("Register dump : {}", registers);
                break;
            }
        }

        // Process delay / sound timer decrasement.
        // Unlike instruction parsing and update, timer must be processed independently.
        // Even machine state is being waited for key input, timer will be processed.
        use engine::register::TimerSideEffect;
        match registers.update_timers() {
            TimerSideEffect::None => (),
            TimerSideEffect::Beep => { beep(); () }
        }

        // Terminate local frame states.
        // Keypad reset should also be processed independently.
        keypad.reset_all();
    }   // End of one frame.

    endwin();
}
