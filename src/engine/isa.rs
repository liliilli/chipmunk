use std::mem;

#[derive(Debug)]
pub enum Instruction {
    Ignore,                         // 0x0nnn SYS addr (IGNORED)
    ClearDisplay,                   // 0x00E0 CLS
    ReturnSubroutine,               // 0x00EE RET
    JmpAddr(u16),                   // 0x1nnn JP Addr Jump to location nnn (program counter).
    CallSub(u16),                   // 0x2nnn CALL addr Call subroutine of nnn with push now ps.
    SkipEq{ r: u8, val: u8 },       // 0x3xkk SE Vx, byte Skip next instruction if Vx == kk.
    SkipNeq{ r: u8, val: u8 },      // 0x4xkk SNE Vx, byte Skip next instruction if Vx != kk.
    SkipRegEq{ r: u8, f: u8 },      // 0x5xy0 SE Vx, Vy. Skip next instruction if Vx == Vy.
    SetByte{ r: u8, val: u8 },      // 0x6xkk LD Vx as r, byte(0xkk) as val
    AddByte{ r: u8, val: u8 },      // 0x7xkk ADD Vx, byte(0xkk) as val, Vx += val
    SetRegV{ r: u8, f: u8 },        // 0x8xy0 LD Vx, Vy Set Vx = Vy.
    OrRegV{ r: u8, f: u8 },         // 0x8xy1 OR Vx, Vy Set Vx |= Vy.
    AndRegV{ r: u8, f: u8 },        // 0x8xy2 AND Vx, Vy Set Vx &= Vy.
    XorRegV{ r: u8, f: u8 },        // 0x8xy3 XOR Vx, Vy Set Vx ^= Vy.
    AddRegV{ r: u8, f: u8 },        // 0x8xy4 ADD Vx, Vy and VF = true if overflow.
    SubRegV{ r: u8, f: u8 },        // 0x8xy5 SUB Vx, Vy and if Vx > Vy, VF = true.
    ShrRegV{ r: u8, f: u8 },        // 0x8xy6 If LSB of Vx is 1, VF = true and Vx >>= Vy.
    SubNRegV{ r: u8, f: u8 },       // 0x8xy7 SUBN Vx, Vy , Vx = Vy - Vx and if Vy > Vx, VF = true.
    ShlRegV{ r: u8, f: u8 },        // 0x8xyE Vx's MSB is 1, VF = true and Vx <<= Vy.
    SkipRegNeq{ r: u8, f: u8 },     // 0x9xy0 SNE Vx, Vy
    SetRegL(u16),                   // 0xAnnn LD l, addr(nnn)
    JmpAddrOffReg0(u16),            // 0xBnnn JP V0, addr(nnn), PC = V0 + nnn.
    RndAnd{ r: u8, val: u8 },       // 0xCxkk RND Vx as r, byte(0xkk) random byte AND kk as val.
    DispSpr{ rp: (u8, u8), n: u8 }, // 0xDxyn DRW Vx, Vy, n-byte sprite with xor from l with xor.
    SkipKeyPressed{ r: u8 },        // 0xEx9E Skip next instruction if VX value key is pressed.
    SkipKeyReleased{ r: u8 },       // 0xExA1 Skip next instruction if VX value key is not pressed.
    SetDelayToReg{ r: u8 },         // 0xFx07 Store the current value of the delay timer to VX.
    WaitKeyPress{ r: u8 },          // 0xFx0A Wait for key press. Pressed key value stored to VX.
    SetDelayFromReg{ r: u8 },       // 0xFx15 Set the delay timer to the value of register VX.
    SetSoundFromReg{ r: u8 },       // 0xFx18 Set the sound timer to the value of register VX.
    AddRegL{ r: u8 },               // 0xFx1E ADD l, Vx. l += Vx.
    SetRegLFontAddrFromReg{ r: u8 },// 0xFx29 Set L to the memory addr from sprite value from VX.
    MemDumpBcdFromReg{ r: u8 },     // 0xFx33 Store BCD from value of VX at address [L, max L+2].
    MemDump{ endr: u8 },            // 0xFx55 LD [l], Vx. Store [V0, Vx] value from [l, l+(x-0)].
    MemRead{ endr: u8 },            // 0xFx65 LD Vx, [l]. Read value from [l, l+(x-0)] to [V0, Vx].
}

fn get_12bit_from(bytes: &[u8; 2]) -> u16 {
    (((bytes[0] & 0x0F) as u16) << 8) + bytes[1] as u16
}

pub fn parse_instruction(bytes: &[u8; 2]) -> Option<Instruction> {
    let opr0 = bytes[0] >> 4;   // Get 0xXX00____ from bytes
    let r = bytes[0] & 0x0F;    // Get 0x__XX____ from bytes
    let val = bytes[1];         // Get 0x____XXXX from bytes

    match opr0 {
        0x0 => {
            match r {
                0 if bytes[1] == 0xE0 => Some(Instruction::ClearDisplay),       // 0x00E0
                0 if bytes[1] == 0xEE => Some(Instruction::ReturnSubroutine),   // 0x00EE
                _ => Some(Instruction::Ignore),
            }
        },
        0x1 => Some(Instruction::JmpAddr(get_12bit_from(bytes))),               // 0x1NNN
        0x2 => Some(Instruction::CallSub(get_12bit_from(bytes))),               // 0x2NNN
        0x3 => Some(Instruction::SkipEq{ r, val }),                             // 0x3XNN
        0x4 => Some(Instruction::SkipNeq{ r, val }),                            // 0x4XNN
        0x5 => Some(Instruction::SkipRegEq{ r, f: bytes[1] >> 4 }),             // 0x5XY0
        0x6 => Some(Instruction::SetByte{ r, val: bytes[1] }),                  // 0x6XNN
        0x7 => Some(Instruction::AddByte{ r, val: bytes[1] }),                  // 0x7XNN
        0x8 => {
            let f = bytes[1] >> 4;
            match bytes[1] & 0x0F {
                0x0 => Some(Instruction::SetRegV{ r, f }),  // 0x8XY0
                0x1 => Some(Instruction::OrRegV{ r, f }),   // 0x8XY1
                0x2 => Some(Instruction::AndRegV{ r, f }),  // 0x8XY2
                0x3 => Some(Instruction::XorRegV{ r, f }),  // 0x8XY3
                0x4 => Some(Instruction::AddRegV{ r, f }),  // 0x8XY4 Carryable
                0x5 => Some(Instruction::SubRegV{ r, f }),  // 0x8XY5 Borrowable
                0x6 => Some(Instruction::ShrRegV{ r, f }),  // 0x8XY6
                0x7 => Some(Instruction::SubNRegV{ r, f }), // 0x8XY7 Borrowable
                0xE => Some(Instruction::ShlRegV{ r, f }),  // 0x8XYE
                _ => None, 
            }
        },
        0x9 => Some(Instruction::SkipRegNeq{ r, f: bytes[1] >> 4 }),            // 0x9XY0
        0xA => Some(Instruction::SetRegL(get_12bit_from(bytes))),               // 0xANNN
        0xB => Some(Instruction::JmpAddrOffReg0(get_12bit_from(bytes))),        // 0xBNNN
        0xC => Some(Instruction::RndAnd{ r, val }),                             // 0xCXNN
        0xD => {                                                                // 0xDXYN
            let rx = r;
            let ry = bytes[1] >> 4;
            Some(Instruction::DispSpr{ rp: (rx, ry), n: bytes[1] & 0x0F })
        },
        0xE => {
            match bytes[1] {
                0x9E => Some(Instruction::SkipKeyPressed{ r }),
                0xA1 => Some(Instruction::SkipKeyReleased{ r }),
                _ => None,
            }
        },
        0xF => {
            match bytes[1] {
                0x07 => Some(Instruction::SetDelayToReg{ r }),
                0x0A => Some(Instruction::WaitKeyPress{ r }),
                0x15 => Some(Instruction::SetDelayFromReg{ r }),
                0x18 => Some(Instruction::SetSoundFromReg{ r }),
                0x1E => Some(Instruction::AddRegL{ r }),
                0x29 => Some(Instruction::SetRegLFontAddrFromReg{ r }),
                0x33 => Some(Instruction::MemDumpBcdFromReg{ r }),
                0x55 => Some(Instruction::MemDump{ endr: r }),
                0x65 => Some(Instruction::MemRead{ endr: r }),
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn to_bitfield_string(bytes: &[u8; 2], true_char: char, false_char: char) -> String {
    static LEN: usize = mem::size_of::<u8>() * 8 * 2;

    let mut result = String::with_capacity(LEN + 1);

    for item in bytes {
        for i in (0..8).rev() {
            if (*item & ((0b1 as u8) << i)) != 0x00 {
                result.push(true_char);
            } else {
                result.push(false_char);
            }
        }

        result.push(' ');
    }
    result.remove(result.len() - 1);
    result
}