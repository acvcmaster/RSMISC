use std::fmt::Display;

use arithmetic_operation::ArithmeticOperation;
use instruction::Instruction;
use operand::Operand;

pub mod arithmetic_operation;
pub mod instruction;
pub mod operand;

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

    pub fn load_32(&self, address: u16) -> Result<u32, RsmiscError> {
        let b0 = self.memory[(address + 0) as usize] as u32;
        let b1 = self.memory[(address + 1) as usize] as u32;
        let b2 = self.memory[(address + 2) as usize] as u32;
        let b3 = self.memory[(address + 3) as usize] as u32;

        Ok(b3 | b2 << 8 | b1 << 16 | b0 << 24)
    }

    pub fn store_32(&mut self, address: u16, dword: u32) -> Result<(), RsmiscError> {
        let b0 = (dword & 0xff000000) >> 24;
        let b1 = (dword & 0xff0000) >> 16;
        let b2 = (dword & 0xff00) >> 8;
        let b3 = (dword & 0xff) >> 0;

        self.memory[(address + 0) as usize] = b0 as u8;
        self.memory[(address + 1) as usize] = b1 as u8;
        self.memory[(address + 2) as usize] = b2 as u8;
        self.memory[(address + 3) as usize] = b3 as u8;

        Ok(())
    }

    pub fn load_16(&self, address: u16) -> Result<u16, RsmiscError> {
        let b0 = self.memory[(address + 0) as usize] as u16;
        let b1 = self.memory[(address + 1) as usize] as u16;

        Ok(b1 | b0 << 8)
    }

    pub fn store_16(&mut self, address: u16, value: u16) -> () {
        let b0 = (value & 0xff00) >> 8;
        let b1 = value & 0xff;

        self.memory[(address + 0) as usize] = b0 as u8;
        self.memory[(address + 1) as usize] = b1 as u8;
    }

    pub fn execute_next(&mut self, print: bool) -> Result<bool, RsmiscError> {
        let result = self.load_32(self.ip);
        match result {
            Ok(dword) => {
                let instruction = Instruction::from(dword);
                self.ip += 0x4;

                match instruction.op_code {
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
                }
            }
            Err(error) => Err(error),
        }
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
        let target_result = self.get_operand_value(instruction, instruction.target);
        let source_result = self.get_operand_value(instruction, instruction.source);

        match (target_result, source_result) {
            (Ok(target), Ok(source)) => match operation {
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
            },
            (Ok(_), Err(source)) => Err(source),
            (Err(target), Ok(_)) => Err(target),
            (Err(target), Err(_)) => Err(target),
        }
    }

    pub fn mov(&mut self, instruction: Instruction, print: bool) -> Result<bool, RsmiscError> {
        if print {
            self.print_instruction(&instruction);
        }

        let source_value = self.get_operand_value(instruction, instruction.source);
        let invalid_move_target = Err(RsmiscError {
            code: -6,
            message: format!("INVALID_MOVE_TARGET (at 0x{:x}", self.ip - 0x4),
        });

        match source_value {
            Ok(source) => match instruction.target {
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
            },
            Err(error) => Err(error),
        }
    }

    pub fn ld(&mut self, instruction: Instruction, print: bool) -> Result<bool, RsmiscError> {
        if print {
            self.print_instruction(&instruction);
        }

        let target_value = self.get_operand_value(instruction, instruction.target);

        match target_value {
            Ok(target) => {
                self.stack.push(target);
                Ok(true)
            }
            Err(error) => Err(error),
        }
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
                    message: format!("INVALID_UNLOAD_TARGET (at 0x{:x}", self.ip - 0x4),
                }),
                Operand::MA => {
                    self.store_16(instruction.imm, value);
                    Ok(true)
                }
            },
            None => Err(RsmiscError {
                code: -4,
                message: format!("NO_ELEMENTS_IN_STACK (at 0x{:x}", self.ip - 0x4),
            }),
        }
    }

    pub fn bz(&mut self, instruction: Instruction, print: bool) -> Result<bool, RsmiscError> {
        if print {
            self.print_instruction(&instruction);
        }

        let target_value = self.get_operand_value(instruction, instruction.target);
        let source_value = self.get_operand_value(instruction, instruction.source);

        match (target_value, source_value) {
            (Ok(target), Ok(source)) => {
                if target == 0 {
                    self.ip = source;
                }

                Ok(true)
            }
            (Ok(_), Err(source)) => Err(source),
            (Err(target), Ok(_)) => Err(target),
            (Err(target), Err(_)) => Err(target),
        }
    }

    pub fn swi(&self, instruction: Instruction, print: bool) -> Result<bool, RsmiscError> {
        if print {
            self.print_instruction(&instruction);
        }

        match instruction.imm {
            0x0 => {
                if let Some(printable) = char::from_u32(self.registers[0] as u32) {
                    print!("{}", printable);
                }
                Ok(true)
            },
            _ => Err(RsmiscError {
                code: -3,
                message: format!(
                    "UNIMPLEMENTED_SOFTWARE_INTERRUPT (at 0x{:x})",
                    self.ip - 0x4
                ),
            })
        }
    }

    pub fn call(&mut self, instruction: Instruction, print: bool) -> Result<bool, RsmiscError> {
        if print {
            self.print_instruction(&instruction);
        }

        let target_value = self.get_operand_value(instruction, instruction.target);
        return match target_value {
            Ok(target) => {
                self.call_stack.push(self.ip);
                self.ip = target;
                Ok(true)
            }
            Err(error) => Err(error),
        };
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
                message: format!("CALL_STACK_EMPTY (at 0x{:x})", self.ip - 0x4),
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
        operand: Operand,
    ) -> Result<u16, RsmiscError> {
        match operand {
            Operand::R1 => Ok(self.registers[0] as u16),
            Operand::R2 => Ok(self.registers[1] as u16),
            Operand::R3 => Ok(self.registers[2] as u16),
            Operand::R4 => Ok(self.registers[3] as u16),
            Operand::IP => Ok(self.ip),
            Operand::CT => Ok(instruction.imm),
            Operand::MA => self.load_16(instruction.imm),
        }
    }

    pub fn print_instruction(&self, instruction: &Instruction) {
        println!("0x{:x}:\t {}", self.ip - 4, instruction);
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
