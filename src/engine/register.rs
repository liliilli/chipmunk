use std::fmt;

extern crate rand;
use super::isa;

/// @brief
const GENERAL_REGISTERS_CNT: usize = 16usize;
const INIT_PROGRAM_COUNTER_VAL: u16 = 0x200u16;

pub enum SideEffect {
    Draw{ pos: (u8, u8), n: u8, l: u16 },   // 
    ClearDisplay,                           // 
    ReturnSubroutine,                       // 
}

pub struct Registers {
    g: [u8; GENERAL_REGISTERS_CNT], // General purpose registers
    sl: u16,                        // Memory address register from SL.
    vf: bool,                       // Flag instruction register (carry & borrow, collision).
    pc: u16,                        // Program counter register.
    sp: u16,                        // Present stack pointer register.
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
            sp: INIT_PROGRAM_COUNTER_VAL,
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

    pub fn update_registers(&mut self, inst: isa::Instruction) -> Option<SideEffect> {
        type Inst = isa::Instruction;
        match inst {
        Inst::Ignore => {
            self.increase_pc(1);
            None
        },
        Inst::ClearDisplay => {
            self.increase_pc(1);
            Some(SideEffect::ClearDisplay)
        }
        Inst::ReturnSubroutine => {
            Some(SideEffect::ReturnSubroutine)
        }
        Inst::JmpAddr(addr) => {
            self.set_pc(addr);
            None
        },
        Inst::CallSub(_) => {
            None
        },
        Inst::SkipEq{ r, val } => {
            if self.g[r as usize] == val { 
                self.increase_pc(2);
            } else { 
                self.increase_pc(1);
            }
            None
        },
        Inst::SkipNeq{ r, val } => {
            if self.g[r as usize] != val { 
                self.increase_pc(2);
            } else { 
                self.increase_pc(1);
            }
            None
        },
        Inst::SetByte{ r, val } => {
            self.g[r as usize] = val;
            self.increase_pc(1);
            None
        },
        Inst::AddByte{ r, val } => {
            self.g[r as usize] += val;
            self.increase_pc(1);
            None
        },
        Inst::SetRegV{ r, f } => {
            self.g[r as usize] = self.g[f as usize];
            self.increase_pc(1);
            None
        },
        Inst::SetRegL(new_l) => {
            self.sl = new_l;
            self.increase_pc(1);
            None
        },
        Inst::RndAnd{ r, val } => {
            self.g[r as usize] = rand::random::<u8>() & val;
            self.increase_pc(1);
            None
        },
        Inst::DispSpr{ rp, n } => {
            let px = self.g[rp.0 as usize];
            let py = self.g[rp.1 as usize];

            self.increase_pc(1);
            Some(SideEffect::Draw{pos: (px, py), n, l: self.sl})
        }
        }
    }

    pub fn proceed_program_counter(&mut self, val: u16) {
        self.pc += val;
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
            "L:{:4},PC:{:4},SP:{:4},DT:{:4},ST:{:4},VF:{}",
            self.sl, self.pc, self.sp, self.dt, self.st, self.vf);

        write!(f, "{}\n{}\n{}", general_registers0, general_registers1, others)
    }
}
