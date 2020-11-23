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
    SetByte{ r: u8, val: u8 },      // 0x6xkk LD Vx as r, byte(0xkk) as val
    AddByte{ r: u8, val: u8 },      // 0x7xkk ADD Vx, byte(0xkk) as val, Vx += val
    SetRegV{ r: u8, f: u8 },        // 0x8xy0 LD Vx, Vy Set Vx = Vy.
    SetRegL(u16),                   // 0xAnnn LD l, addr(nnn)
    RndAnd{ r: u8, val: u8 },       // 0xCxkk RND Vx as r, byte(0xkk) random byte AND kk as val.
    DispSpr{ rp: (u8, u8), n: u8 }, // 0xDxyn DRW Vx, Vy, n-byte sprite with xor from l with xor.
}

pub fn parse_instruction(bytes: &[u8; 2]) -> Option<Instruction> {
    let opr0 = bytes[0] >> 4;
    let r= bytes[0] & 0x0F;
    let val = bytes[1];

    match opr0 {
        0x00 => {
            match r {
                0 if bytes[1] == 0xE0 => Some(Instruction::ClearDisplay),
                0 if bytes[1] == 0xEE => Some(Instruction::ReturnSubroutine),
                _ => Some(Instruction::Ignore),
            }
        },
        0x01 => Some(Instruction::JmpAddr(((r as u16) << 8) + bytes[1] as u16)),
        0x02 => Some(Instruction::CallSub(((r as u16) << 8) + bytes[1] as u16)),
        0x03 => Some(Instruction::SkipEq{ r, val }),
        0x04 => Some(Instruction::SkipNeq{ r, val }),
        0x06 => Some(Instruction::SetByte{ r, val }),
        0x07 => Some(Instruction::AddByte{ r, val }),
        0x08 => {
            let f = bytes[1] >> 4;
            match bytes[1] & 0x0F {
                0 => Some(Instruction::SetRegV{ r, f }),
                _ => None, // @todo implement 1,2,3,4,5,6,7,8,9...
            }
        },
        0x0A => Some(Instruction::SetRegL(((r as u16) << 8) + bytes[1] as u16)),
        0x0C => Some(Instruction::RndAnd{ r, val }),
        0x0D => {
            let rx = r;
            let ry = bytes[1] >> 4;
            Some(Instruction::DispSpr{ rp: (rx, ry), n: bytes[1] & 0x0F })
        },
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