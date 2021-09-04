use std::fmt::Debug;

#[derive(Copy, Debug, Clone)]
pub enum Operand {
    R1,
    R2,
    R3,
    R4,
    IP,
    CT,
    MA,
}

pub const OPERAND_COUNT: u8 = 7;

impl Operand {
    pub fn get_combination_target(operand_combination: u8) -> Operand {
        match operand_combination / OPERAND_COUNT {
            0 => Operand::R1,
            1 => Operand::R2,
            2 => Operand::R3,
            3 => Operand::R4,
            4 => Operand::IP,
            5 => Operand::CT,
            6 => Operand::MA,
            _ => Operand::R1, // fallback
        }
    }

    pub fn get_combination_source(operand_combination: u8) -> Operand {
        match operand_combination % OPERAND_COUNT {
            0 => Operand::R1,
            1 => Operand::R2,
            2 => Operand::R3,
            3 => Operand::R4,
            4 => Operand::IP,
            5 => Operand::CT,
            6 => Operand::MA,
            _ => Operand::R1, // fallback
        }
    }

    pub fn display(&self, imm: u16) -> String {
        match self {
            Operand::CT => format!("#{:X}", imm),
            Operand::MA => format!("{:X}", imm),
            _ => format!("{:?}", self)
        }
    }
}   