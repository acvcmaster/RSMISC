use std::fmt::Display;

use arithmetic_operation::ArithmeticOperation;
use instruction::Instruction;
use operand::{Operand, OperandType};

pub mod arithmetic_operation;
pub mod instruction;
pub mod operand;

// Size (in bytes) of the instruction
const INSTRUCTION_SIZE: usize = 0x6;

#[derive(Debug, Clone)]
pub struct Rsmisc {
    memory: [u8; 0xffff], // 64 KiB
    ip: u16,
    registers: [u16; 0x4], // R1, R2, R3, R4,
    stack: Vec<u16>,
    call_stack: Vec<u16>,
}

impl Rsmisc {
    pub fn new(program: &Vec<u8>) -> Result<Self, RsmiscError> {
        let mut result = Self {
            memory: [0; 0xffff],
            ip: 0,
            registers: [0; 0x4],
            stack: Vec::new(),
            call_stack: Vec::new(),
        };
        let length = program.len();

        for address in 0..length {
            result.memory[address] = program[address];
        }

        Ok(result)
    }

    pub fn load_48(&self, address: u16) -> Result<u64, RsmiscError> {
        let mut result = 0;

        for index in 0..INSTRUCTION_SIZE {
            let current = self.memory[address as usize + index] as u64;
            result |= current << (INSTRUCTION_SIZE - (index + 1)) * 8;
        }

        Ok(result)
    }

    pub fn load_16(&self, address: u16) -> Result<u16, RsmiscError> {
        let mut result = 0;

        for index in 0..2 {
            let current = self.memory[address as usize + index] as u16;
            result |= current << (2 - (index + 1)) * 8;
        }

        Ok(result)
    }

    pub fn store_16(&mut self, address: u16, value: u16) -> () {
        let b0 = (value & 0xff00) >> 8;
        let b1 = value & 0xff;

        self.memory[(address + 0) as usize] = b0 as u8;
        self.memory[(address + 1) as usize] = b1 as u8;
    }

    pub fn execute_next(&mut self, print: bool) -> Result<bool, RsmiscError> {
        // Fetch the instruction
        let instruction = Instruction::from(self.load_48(self.ip)?);

        // Execute the instruction
        let result = match instruction.op_code {
            instruction::Opcode::HALT => self.halt(instruction, print),
            instruction::Opcode::ADD => self.add(instruction, print),
            instruction::Opcode::SUB => self.sub(instruction, print),
            instruction::Opcode::MUL => self.mul(instruction, print),
            instruction::Opcode::DIV => self.div(instruction, print),
            instruction::Opcode::MOV => self.mov(instruction, print),
            instruction::Opcode::LD => self.ld(instruction, print),
            instruction::Opcode::ULD => self.uld(instruction, print),
            instruction::Opcode::BZ => self.bz(instruction, print),
            instruction::Opcode::SWI => self.swi(instruction, print),
            instruction::Opcode::CALL => self.call(instruction, print),
            instruction::Opcode::RET => self.ret(instruction, print),
            instruction::Opcode::NOP => self.nop(instruction, print),
        };

        // Increment the instruction pointer
        self.ip += INSTRUCTION_SIZE as u16;

        return result;
    }

    pub fn halt(&self, instruction: Instruction, print: bool) -> Result<bool, RsmiscError> {
        if print {
            self.print_instruction(&instruction);
        }

        Ok(false)
    }

    pub fn add(&mut self, instruction: Instruction, print: bool) -> Result<bool, RsmiscError> {
        if print {
            self.print_instruction(&instruction);
        }

        self.arithmetic_operation(instruction, ArithmeticOperation::Add)
    }

    pub fn sub(&mut self, instruction: Instruction, print: bool) -> Result<bool, RsmiscError> {
        if print {
            self.print_instruction(&instruction);
        }

        self.arithmetic_operation(instruction, ArithmeticOperation::Sub)
    }

    pub fn mul(&mut self, instruction: Instruction, print: bool) -> Result<bool, RsmiscError> {
        if print {
            self.print_instruction(&instruction);
        }

        self.arithmetic_operation(instruction, ArithmeticOperation::Mul)
    }

    pub fn div(&mut self, instruction: Instruction, print: bool) -> Result<bool, RsmiscError> {
        if print {
            self.print_instruction(&instruction);
        }

        self.arithmetic_operation(instruction, ArithmeticOperation::Div)
    }

    pub fn arithmetic_operation(
        &mut self,
        instruction: Instruction,
        operation: ArithmeticOperation,
    ) -> Result<bool, RsmiscError> {
        let target = self.get_operand_value(instruction, OperandType::TARGET)?;
        let source = self.get_operand_value(instruction, OperandType::SOURCE)?;

        match operation {
            ArithmeticOperation::Add => {
                self.stack.push(target.wrapping_add(source));
                Ok(true)
            }
            ArithmeticOperation::Sub => {
                self.stack.push(target.wrapping_sub(source));
                Ok(true)
            }
            ArithmeticOperation::Mul => {
                self.stack.push(target.wrapping_mul(source));
                Ok(true)
            }
            ArithmeticOperation::Div => {
                self.stack.push(target.wrapping_div(source));
                Ok(true)
            }
        }
    }

    pub fn mov(&mut self, instruction: Instruction, print: bool) -> Result<bool, RsmiscError> {
        if print {
            self.print_instruction(&instruction);
        }

        let source = self.get_operand_value(instruction, OperandType::SOURCE)?;
        let invalid_move_target = Err(RsmiscError {
            code: -6,
            message: format!("INVALID_MOVE_TARGET (at 0x{:x}", self.ip),
        });

        match instruction.target {
            Operand::R1 => {
                self.registers[0] = source;
                Ok(true)
            }
            Operand::R2 => {
                self.registers[1] = source;
                Ok(true)
            }
            Operand::R3 => {
                self.registers[2] = source;
                Ok(true)
            }
            Operand::R4 => {
                self.registers[3] = source;
                Ok(true)
            }
            Operand::IP => {
                self.ip = source;
                Ok(true)
            }
            Operand::CT => invalid_move_target,
            Operand::MA => invalid_move_target,
        }
    }

    pub fn ld(&mut self, instruction: Instruction, print: bool) -> Result<bool, RsmiscError> {
        if print {
            self.print_instruction(&instruction);
        }

        self.stack
            .push(self.get_operand_value(instruction, OperandType::TARGET)?);

        Ok(true)
    }

    pub fn uld(&mut self, instruction: Instruction, print: bool) -> Result<bool, RsmiscError> {
        if print {
            self.print_instruction(&instruction);
        }

        match self.stack.pop() {
            Some(value) => match instruction.target {
                Operand::R1 => {
                    self.registers[0] = value;
                    Ok(true)
                }
                Operand::R2 => {
                    self.registers[1] = value;
                    Ok(true)
                }
                Operand::R3 => {
                    self.registers[2] = value;
                    Ok(true)
                }
                Operand::R4 => {
                    self.registers[3] = value;
                    Ok(true)
                }
                Operand::IP => {
                    self.ip = value;
                    Ok(true)
                }
                Operand::CT => Err(RsmiscError {
                    code: -5,
                    message: format!("INVALID_UNLOAD_TARGET (at 0x{:x}", self.ip),
                }),
                Operand::MA => {
                    self.store_16(instruction.target_imm, value);
                    Ok(true)
                }
            },
            None => Err(RsmiscError {
                code: -4,
                message: format!("NO_ELEMENTS_IN_STACK (at 0x{:x}", self.ip),
            }),
        }
    }

    pub fn bz(&mut self, instruction: Instruction, print: bool) -> Result<bool, RsmiscError> {
        if print {
            self.print_instruction(&instruction);
        }

        let target = self.get_operand_value(instruction, OperandType::TARGET)?;
        let source = self.get_operand_value(instruction, OperandType::SOURCE)?;

        if target == 0 {
            self.ip = source;
        }

        Ok(true)
    }

    pub fn swi(&self, instruction: Instruction, print: bool) -> Result<bool, RsmiscError> {
        if print {
            self.print_instruction(&instruction);
        }

        match instruction.target_imm {
            0x0 => self.print_character_swi(),
            0x1 => self.print_number_swi(),
            _ => Err(RsmiscError {
                code: -3,
                message: format!("UNIMPLEMENTED_SOFTWARE_INTERRUPT (at 0x{:x})", self.ip),
            }),
        }
    }

    fn print_character_swi(&self) -> Result<bool, RsmiscError> {
        if let Some(printable) = char::from_u32(self.registers[0] as u32) {
            print!("{}", printable);
        }

        Ok(true)
    }

    fn print_number_swi(&self) -> Result<bool, RsmiscError> {
        print!("{}", self.registers[0]);

        Ok(true)
    }

    pub fn call(&mut self, instruction: Instruction, print: bool) -> Result<bool, RsmiscError> {
        if print {
            self.print_instruction(&instruction);
        }

        let target = self.get_operand_value(instruction, OperandType::TARGET)?;

        self.call_stack.push(self.ip + (INSTRUCTION_SIZE as u16));
        self.ip = target - (INSTRUCTION_SIZE as u16);

        Ok(true)
    }

    pub fn ret(&mut self, instruction: Instruction, print: bool) -> Result<bool, RsmiscError> {
        if print {
            self.print_instruction(&instruction);
        }

        match self.call_stack.pop() {
            Some(value) => {
                self.ip = value;
                Ok(true)
            }
            None => Err(RsmiscError {
                code: -2,
                message: format!("CALL_STACK_EMPTY (at 0x{:x})", self.ip),
            }),
        }
    }

    pub fn nop(&self, instruction: Instruction, print: bool) -> Result<bool, RsmiscError> {
        if print {
            self.print_instruction(&instruction);
        }

        Ok(true)
    }

    pub fn get_operand_value(
        &self,
        instruction: Instruction,
        operand_type: OperandType,
    ) -> Result<u16, RsmiscError> {
        let operand = match operand_type {
            OperandType::TARGET => instruction.target,
            OperandType::SOURCE => instruction.source,
        };
        let immediate = match operand_type {
            OperandType::TARGET => instruction.target_imm,
            OperandType::SOURCE => instruction.source_imm,
        };

        match operand {
            Operand::R1 => Ok(self.registers[0] as u16),
            Operand::R2 => Ok(self.registers[1] as u16),
            Operand::R3 => Ok(self.registers[2] as u16),
            Operand::R4 => Ok(self.registers[3] as u16),
            Operand::IP => Ok(self.ip),
            Operand::CT => Ok(immediate),
            Operand::MA => self.load_16(immediate),
        }
    }

    pub fn print_instruction(&self, instruction: &Instruction) {
        println!("0x{:x}:\t {}", self.ip, instruction);
    }
}

impl Display for Rsmisc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let call_stack = if let Some(last) = self.call_stack.last() {
            format!("CS: 0x{:x}\tCSP: 0x{:x}", last, self.call_stack.len())
        } else {
            format!("CS: -\t\tCSP: 0x0")
        };

        let stack = if let Some(last) = self.stack.last() {
            format!("S: 0x{:x}\tSP: 0x{:x}", last, self.stack.len())
        } else {
            format!("S: -\t\tSP: 0x0")
        };

        let r1_r2 = format!(
            "R1: 0x{:x}\tR2: 0x{:x}",
            self.registers[0], self.registers[1]
        );

        let r3_r4 = format!(
            "R3: 0x{:x}\tR4: 0x{:x}",
            self.registers[2], self.registers[3]
        );

        let ip = format!("IP: 0x{:x}", self.ip);

        write!(
            f,
            "| Machine state\n|\n| {}\n| {}\n| {}\n| {}\n| {}",
            call_stack, stack, r1_r2, r3_r4, ip
        )
    }
}

#[derive(Debug, Clone)]
pub struct RsmiscError {
    pub code: i32,
    pub message: String,
}
