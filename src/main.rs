use std::env;
use std::time;

mod engine;
use engine::register::{Registers};
use engine::memory::{Memory};
use engine::screen::{Screen, DrawMessage, PixelState};
use engine::keypad::Keypad;
use engine::state::MachineState;
use engine::check::get_ch8_file_path;
use engine::device;
use engine::timer;

extern crate crossterm;
use crossterm::event::{poll, read, Event, KeyEvent, KeyCode};

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
    let mut clock = timer::Timer::from_second(1.0 / 1_760_000.0);
    let mut timer_60hz = timer::Timer::from_second(1.0 / 60.0);

    // Set ncurse window (Render & keyboard input)
    let device = device::Device::new();
    if let Err(err) = device {
        println!("Error : {:?}", err);
        return;
    }
    let mut device = device.unwrap();
    let _ = device.clear();

    // Start one frame.
    loop {
        if clock.tick() == false {
            continue;
        }

        let input_keyval = match poll(time::Duration::from_secs(0)) {
            Ok(true) => {
                // calling read() will be unblocked because some input is already polled.
                match read().unwrap() {
                    // If read value has KeyCode::Char(), try to update keypad state.
                    Event::Key(KeyEvent{ code: KeyCode::Char(val), modifiers: _ }) => {
                        keypad.set_press(val)
                    },
                    // If Escape key is pressed, terminate program.
                    Event::Key(KeyEvent{ code: KeyCode::Esc, modifiers: _ }) => break,
                    _ => None,
                }
            },
            Ok(false) => None,
            _ => break,
        };

        // If machine state is waiting for key press, and some valueable key is pressed,
        // Change machine state and process side-effect.
        if let MachineState::WaitKeyPress{ r } = machine_state {
            if let Some(keyval) = input_keyval {
                registers.set_general_register(r, keyval);
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
                        let _ = device.clear();
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
                        for DrawMessage { pos, state } in &dirty_pixels {
                            match state {
                                PixelState::Erased => { 
                                    let _ = device.mv_print(*pos, " "); 
                                    () 
                                },
                                PixelState::Drawn => { 
                                    let _ = device.mv_print(*pos, "\u{2588}"); 
                                    () 
                                },
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
                        // Let machine wait for new key press.
                        machine_state = MachineState::WaitKeyPress{ r };
                    },
                    Some(SideEffect::CheckKeyPressed{ key }) => {
                        match keypad.check_press(key) {
                            true => registers.increase_pc(2),
                            false => registers.increase_pc(1),
                        }
                    },
                    Some(SideEffect::CheckKeyReleased{ key }) => {
                        match keypad.check_press(key) {
                            false => registers.increase_pc(2),
                            true => registers.increase_pc(1),
                        }
                    },
                    None => (),
                }
            } else { 
                // Failure. Abort program.
                println!("Register dump : {}", registers);
                break;
            }
        }

        // Process delay / sound timer decreasement.
        // Unlike instruction parsing and update, timer must be processed independently.
        // Even machine state is being waited for key input, timer will be processed.
        if timer_60hz.tick() == true {
            use engine::register::TimerSideEffect;
            match registers.update_timers() {
                TimerSideEffect::None => (),
                TimerSideEffect::Beep => (),
            }
        }

        // Terminate local frame states.
        // Keypad reset should also be processed independently.
        keypad.reset_all();
    }   // End of one frame.
}
