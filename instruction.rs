use std::fmt::Display;

use crate::operand::Operand;

#[derive(Copy, Debug, Clone)]
pub enum Opcode {
    HALT,
    ADD,
    SUB,
    MUL,
    DIV,
    MOV,
    LD,
    ULD,
    BZ,
    SWI,
    CALL,
    RET,
    NOP,
}

#[derive(Copy, Debug, Clone)]
pub struct Instruction {
    pub op_code: Opcode,
    pub target: Operand,
    pub source: Operand,
    pub imm: u16,
}

impl From<u32> for Instruction {
    fn from(dword: u32) -> Self {
        let op = ((dword & 0xff000000) >> 24) as u8;
        let operand_combination = ((dword & 0xff0000) >> 16) as u8;
        let imm_1 = dword & 0xff00;
        let imm_2 = dword & 0xff;

        Instruction {
            op_code: match op {
                0x0 => Opcode::HALT,
                0x1 => Opcode::ADD,
                0x2 => Opcode::SUB,
                0x3 => Opcode::MUL,
                0x4 => Opcode::DIV,
                0x5 => Opcode::MOV,
                0x6 => Opcode::LD,
                0x7 => Opcode::ULD,
                0x8 => Opcode::BZ,
                0x9 => Opcode::SWI,
                0xA => Opcode::CALL,
                0xB => Opcode::RET,
                _ => Opcode::NOP,
            },
            target: Operand::get_combination_target(operand_combination),
            source: Operand::get_combination_source(operand_combination),
            imm: (imm_1 >> 8 | imm_2 << 8) as u16,
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.op_code {
            Opcode::HALT => write!(f, "HALT"),
            Opcode::ADD => write!(f, "ADD {} {}", self.target.display(self.imm), self.source.display(self.imm)),
            Opcode::SUB => write!(f, "SUB {} {}", self.target.display(self.imm), self.source.display(self.imm)),
            Opcode::MUL => write!(f, "MUL {} {}", self.target.display(self.imm), self.source.display(self.imm)),
            Opcode::DIV => write!(f, "DIV {} {}", self.target.display(self.imm), self.source.display(self.imm)),
            Opcode::MOV => write!(f, "MOV {} {}", self.target.display(self.imm), self.source.display(self.imm)),
            Opcode::LD => write!(f, "LD {}", self.target.display(self.imm)),
            Opcode::ULD => write!(f, "ULD {}", self.target.display(self.imm)),
            Opcode::BZ => write!(f, "BZ {} {}", self.target.display(self.imm), self.source.display(self.imm)),
            Opcode::SWI => write!(f, "SWI {}", self.target.display(self.imm)),
            Opcode::CALL => write!(f, "CALL {}", self.target.display(self.imm)),
            Opcode::RET => write!(f, "RET"),
            Opcode::NOP => write!(f, "NOP"),
        }
    }
}