use std::char;

/// Provides CHIP-8 COSMAX VIP simulated keypad.
/// The CHIP-8 interpreter will accept input from a 16-key keypad.
pub struct Keypad {
    keypad: [bool; 16],
}

impl Keypad {
    /// Crate new keypad instance.
    pub fn new() -> Keypad {
        Keypad {
            keypad: [false; 16]
        }
    }

    /// Reset all keypad state into not-pressed.
    pub fn reset_all(&mut self) {
        for item in self.keypad.iter_mut() {
            *item = false;
        }
    }

    /// Set matched key from given 'chr' to pressed state. 
    /// If any matched key is not found, do nothing.
    /// Given 'chr' input must be alphabetic or keyboard 1, 2, 3, or 4.
    pub fn set_press(&mut self, chr: char) {
        if chr.is_alphanumeric() == false {
            return;
        }

        // マッチング方法がC++側からみたらこれじゃないようだけど、別のもっと簡単な方法があるだろうか…
        match &chr.to_lowercase().to_string()[..] {
            "x" => { self.keypad[0x0] = true; () },
            "1" => { self.keypad[0x1] = true; () },
            "2" => { self.keypad[0x2] = true; () },
            "3" => { self.keypad[0x3] = true; () },
            "q" => { self.keypad[0x4] = true; () },
            "w" => { self.keypad[0x5] = true; () },
            "e" => { self.keypad[0x6] = true; () },
            "a" => { self.keypad[0x7] = true; () },
            "s" => { self.keypad[0x8] = true; () },
            "d" => { self.keypad[0x9] = true; () },
            "z" => { self.keypad[0xA] = true; () },
            "c" => { self.keypad[0xB] = true; () },
            "4" => { self.keypad[0xC] = true; () },
            "r" => { self.keypad[0xD] = true; () },
            "f" => { self.keypad[0xE] = true; () },
            "v" => { self.keypad[0xF] = true; () },
            _ => (),
        }
    }
}