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
            imm: (imm_1 | imm_2) as u16,
        }
    }
}
