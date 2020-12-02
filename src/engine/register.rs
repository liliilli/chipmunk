use std::fmt;

extern crate rand;
use super::isa;

/// @brief
const GENERAL_REGISTERS_CNT: usize = 16usize;
const STACK_POINTER_CNT: usize = 16usize;
const INIT_PROGRAM_COUNTER_VAL: u16 = 0x200u16;

pub enum SideEffect {
    Draw{ pos: (u8, u8), n: u8, l: u16 },   // 
    ClearDisplay,                           // 
    MemDump{ dump_vals: Vec<u8>, l: u16 },  //
    MemRead{ count: u8, l: u16 },           //
}

pub struct Registers {
    g: [u8; GENERAL_REGISTERS_CNT], // General purpose registers
    sl: u16,                        // Memory address register from SL.
    vf: bool,                       // Flag instruction register (carry & borrow, collision).
    pc: u16,                        // Program counter register.
    spst: Vec<u16>,                 // Stack pointer stack.
    dt: u16,                        // Delay timer register.
    st: u16,                        // Sound timer register.
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            g: [0; GENERAL_REGISTERS_CNT],
            sl: 0,
            vf: false,
            pc: INIT_PROGRAM_COUNTER_VAL,
            spst: Vec::<u16>::with_capacity(STACK_POINTER_CNT),
            dt: 0,
            st: 0,
        }
    }

    pub fn get_pc(&self) -> u16 { self.pc }

    fn set_pc(&mut self, new_pc: u16) {
        self.pc = new_pc;
    }

    fn increase_pc(&mut self, inst_count: u16) {
        self.pc += inst_count << 1;
    }

    pub fn update_vf(&mut self, is_set: bool) {
        self.vf = is_set;
    }

    pub fn update_registers(&mut self, instruction: isa::Instruction) -> Option<SideEffect> {
        type Inst = isa::Instruction;

        let (pc_increment, side_effect) = match instruction {
            Inst::Ignore => (1, None), // 0x0___
            Inst::ClearDisplay => (1, Some(SideEffect::ClearDisplay)), // 0x00E0
            Inst::ReturnSubroutine => { // 0x00EE
                assert!(self.spst.is_empty() == false);
                let new_pc = self.spst.pop().unwrap();
                self.set_pc(new_pc);
                self.increase_pc(1);
                (0, None)
            },
            Inst::JmpAddr(addr) => { // 0x1nnn
                self.set_pc(addr);
                (0, None)
            },
            Inst::JmpAddrOffReg0(new_pc) => { // 0xBnnn
                self.set_pc(self.g[0] as u16 + new_pc);
                (0, None)
            },
            Inst::CallSub(new_pc) => { // 0x2nnn
                self.spst.push(self.get_pc());
                self.set_pc(new_pc);
                (0, None)
            },
            Inst::SkipEq{ r, val } => { // 0x3xkk
                let pc_inc = if self.g[r as usize] == val { 2 } else { 1 };
                (pc_inc, None)
            },
            Inst::SkipNeq{ r, val } => { // 0x4xkk
                let pc_inc = if self.g[r as usize] != val { 2 } else { 1 };
                (pc_inc, None)
            },
            Inst::SkipRegEq{ r, f } => {
                let matched = self.g[r as usize] == self.g[f as usize];
                (if matched { 2 } else { 1 }, None)
            },
            Inst::SetByte{ r, val } => { // 0x6xkk
                self.g[r as usize] = val;
                (1, None)
            },
            Inst::AddByte{ r, val } => { // 0x7xkk
                self.g[r as usize] += val;
                (1, None)
            },
            Inst::SetRegV{ r, f } => { // 0x8xy0
                self.g[r as usize] = self.g[f as usize];
                (1, None)
            },
            Inst::OrRegV{ r, f } => { // 0x8xy1
                self.g[r as usize] |= self.g[f as usize];
                (1, None)
            },
            Inst::AndRegV{ r, f } => { // 0x8xy2
                self.g[r as usize] &= self.g[f as usize];
                (1, None)
            },
            Inst::XorRegV{ r, f } => { // 0x8xy3
                self.g[r as usize] ^= self.g[f as usize];
                (1, None)
            },
            Inst::AddRegV{ r, f } => { // 0x8xy4
                let (val, is_carry) = self.g[r as usize].overflowing_add(self.g[f as usize]);
                self.g[r as usize] = val;
                self.update_vf(is_carry);
                (1, None)
            },
            Inst::SubRegV{ r, f } => { // 0x8xy5
                let (val, is_borrow) = self.g[r as usize].overflowing_sub(self.g[f as usize]);
                self.g[r as usize] = val;
                self.update_vf(!is_borrow); // in CHIP-8, Vx > Vy, update to 1 as not borrowed.
                (1, None)
            },
            Inst::ShrRegV{ r } => { // 0x8x_6
                self.update_vf((self.g[r as usize] & 0b01) == 0b01);
                self.g[r as usize] >>= 1;
                (1, None)
            },
            Inst::SubNRegV{ r, f } => { // 0x8xy7
                let (val, is_borrow) = self.g[f as usize].overflowing_sub(self.g[r as usize]);
                self.g[r as usize] = val;
                self.update_vf(!is_borrow); // in CHIP-8, Vy > Vx, update to 1 as not borrowed.
                (1, None)
            },
            Inst::ShlRegV{ r } => { // 0x8x_E
                self.update_vf((self.g[r as usize] & 0x80) == 0x80);
                self.g[r as usize] <<= 1;
                (1, None)
            },
            Inst::SkipRegNeq{ r, f } => {
                let matched = self.g[r as usize] != self.g[f as usize];
                (if matched { 2 } else { 1 }, None)
            },
            Inst::SetRegL(new_l) => { // 0xAnnn
                self.sl = new_l;
                (1, None)
            },
            Inst::RndAnd{ r, val } => { // 0xCxkk
                self.g[r as usize] = rand::random::<u8>() & val;
                (1, None)
            },
            Inst::DispSpr{ rp, n } => { // 0xDxyn
                let px = self.g[rp.0 as usize];
                let py = self.g[rp.1 as usize];
                (1, Some(SideEffect::Draw{pos: (px, py), n, l: self.sl}))
            },
            Inst::AddRegL{ r } => { // 0xFx1E
                self.sl += self.g[r as usize] as u16;
                (1, None)
            },
            Inst::MemDump{ endr } => { // 0xFx55
                (1, Some(SideEffect::MemDump{ dump_vals: self.g[0..=endr as usize].to_vec(), l: self.sl }))
            },
            Inst::MemRead{ endr } => { // 0xFx65
                (1, Some(SideEffect::MemRead{ count: endr + 1, l: self.sl }))
            }
        };

        // Increase program counter and return side effect to other module.
        self.increase_pc(pc_increment);
        side_effect
    }

    pub fn store_from_v0(&mut self, values: &[u8]) {
        for (idx, &val) in values.iter().enumerate() {
            self.g[idx] = val;
        }
    }
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> { 
        let general_registers0 = format!(
            "G0:{:3}, G1:{:3}, G2:{:3}, G3:{:3} G4:{:3}, G5:{:3}, G6:{:3}, G7:{:3}", 
            self.g[0], self.g[1], self.g[2], self.g[3], 
            self.g[4], self.g[5], self.g[6], self.g[7]);
        let general_registers1 = format!(
            "G8:{:3}, G9:{:3}, GA:{:3}, GB:{:3} GC:{:3}, GD:{:3}, GE:{:3}, GF:{:3}", 
            self.g[8], self.g[9], self.g[10], self.g[11], 
            self.g[12], self.g[13], self.g[14], self.g[15]);
        let others = format!(
            "L:{:4},PC:{:4},DT:{:4},ST:{:4},VF:{}",
            self.sl, self.pc, self.dt, self.st, self.vf);

        write!(f, "{}, {}\n{}", general_registers0, general_registers1, others)
    }
}
