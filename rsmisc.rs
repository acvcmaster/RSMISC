use crate::debug_symbol::DebugSymbol;

pub mod debug_symbol;

#[derive(Debug, Clone)]
pub struct Rsmisc {
    memory: [u8; 0xffff], // 64 KiB
    ip: u16,
    registers: [u8; 0x4], // R1, R2, R3, R4,
    debug_symbols: Vec<DebugSymbol>,
}

impl Rsmisc {
    pub fn new(program: &Vec<u8>, debug_symbols: &Vec<DebugSymbol>) -> Result<Self, RsmiscError> {
        let mut result = Self {
            memory: [0; 0xffff],
            ip: 0,
            registers: [0; 0x4],
            debug_symbols: debug_symbols.clone(),
        };
        let length = program.len();

        if length % 0x4 != 0 {
            return Err(RsmiscError {
                code: -1,
                message: format!("INVALID_PROGRAM_LENGTH"),
            });
        }

        for address in (0..length).step_by(0x4) {
            let b0 = program[address + 0] as u32;
            let b1 = program[address + 1] as u32;
            let b2 = program[address + 2] as u32;
            let b3 = program[address + 3] as u32;

            if let Err(error) = result.store_32(address as u32, b3 | b2 << 8 | b1 << 16 | b0 << 24)
            {
                // Never going to trigger (step by 4)
                return Err(error);
            }
        }

        Ok(result)
    }

    pub fn load_32(&mut self, address: u32) -> Result<u32, RsmiscError> {
        match address % 0x4 {
            0 => {
                let b0 = self.memory[(address + 0) as usize] as u32;
                let b1 = self.memory[(address + 1) as usize] as u32;
                let b2 = self.memory[(address + 2) as usize] as u32;
                let b3 = self.memory[(address + 3) as usize] as u32;

                Ok(b3 | b2 << 8 | b1 << 16 | b0 << 24)
            }
            _ => Err(RsmiscError {
                code: -2,
                message: format!("UNALIGNED_LOAD_32"),
            }),
        }
    }

    pub fn store_32(&mut self, address: u32, word: u32) -> Result<(), RsmiscError> {
        match address % 0x4 {
            0 => {
                let b0 = (word & 0xff000000) >> 24;
                let b1 = (word & 0xff0000) >> 16;
                let b2 = (word & 0xff00) >> 8;
                let b3 = (word & 0xff) >> 0;

                self.memory[(address + 0) as usize] = b0 as u8;
                self.memory[(address + 1) as usize] = b1 as u8;
                self.memory[(address + 2) as usize] = b2 as u8;
                self.memory[(address + 3) as usize] = b3 as u8;

                Ok(())
            }
            _ => Err(RsmiscError {
                code: -2,
                message: format!("UNALIGNED_STORE_32"),
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RsmiscError {
    pub code: i32,
    pub message: String,
}
