/// Provides global state of CHIP-8 machine.
#[derive(Debug, PartialEq)]
pub enum MachineState {
    Normal,                 // Process machine normally.
    WaitKeyPress{ r: u8 },  // Wait for key press, processing instruction should be paused.
}