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
    pub fn set_press(&mut self, chr: char) -> Option<u8> {
        if chr.is_alphanumeric() == false {
            return None;
        }

        // マッチング方法がC++側からみたらこれじゃないようだけど、別のもっと簡単な方法があるだろうか…
        match &chr.to_lowercase().to_string()[..] {
            "x" => { self.keypad[0x0] = true; Some(0x0u8) },
            "1" => { self.keypad[0x1] = true; Some(0x1u8) },
            "2" => { self.keypad[0x2] = true; Some(0x2u8) },
            "3" => { self.keypad[0x3] = true; Some(0x3u8) },
            "q" => { self.keypad[0x4] = true; Some(0x4u8) },
            "w" => { self.keypad[0x5] = true; Some(0x5u8) },
            "e" => { self.keypad[0x6] = true; Some(0x6u8) },
            "a" => { self.keypad[0x7] = true; Some(0x7u8) },
            "s" => { self.keypad[0x8] = true; Some(0x8u8) },
            "d" => { self.keypad[0x9] = true; Some(0x9u8) },
            "z" => { self.keypad[0xA] = true; Some(0xAu8) },
            "c" => { self.keypad[0xB] = true; Some(0xBu8) },
            "4" => { self.keypad[0xC] = true; Some(0xCu8) },
            "r" => { self.keypad[0xD] = true; Some(0xDu8) },
            "f" => { self.keypad[0xE] = true; Some(0xEu8) },
            "v" => { self.keypad[0xF] = true; Some(0xFu8) },
            _ => None,
        }
    }

    /// Check whether given key is pressed or not.
    /// If key is pressed, return true. Otherwise, return false.
    /// 
    /// If invalid key index that is larger than 0x0F is inputed, 
    /// program will be halted.
    pub fn check_press(&self, key: u8) -> bool {
        assert!(key <= 0xFu8, "");
        self.keypad[key as usize]
    }
}