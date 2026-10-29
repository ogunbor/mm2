//! 8-instruction ISA

pub const NUM_REGS: usize = 5;

/// The eight opcodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    Nop = 0,      // No operation (used for trace padding)
    Imm = 1,      // rd = imm
    Add = 2,      // rd = rs1 + rs2
    Mul = 3,      // rd = rs1 * rs2
    Sub = 4,      // rd = rs1 - rs2
    Halt = 5,     // Stop execution
    Addi = 6,     // rd = rs1 + imm
    AssertEq = 7, // Require rs1 == rs2
}

impl Opcode {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Opcode::Nop,
            1 => Opcode::Imm,
            2 => Opcode::Add,
            3 => Opcode::Mul,
            4 => Opcode::Sub,
            5 => Opcode::Halt,
            6 => Opcode::Addi,
            7 => Opcode::AssertEq,
            _ => panic!("invalid opcode: {}", v),
        }
    }
}

/// A single instruction.
#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub opcode: Opcode,
    pub rd: u8,
    pub rs1: u8,
    pub rs2: u8,
    pub imm: u64,
}

impl Instruction {
    pub fn nop() -> Self {
        Instruction {
            opcode: Opcode::Nop,
            rd: 0,
            rs1: 0,
            rs2: 0,
            imm: 0,
        }
    }

    pub fn imm(rd: u8, value: u64) -> Self {
        assert!((rd as usize) < NUM_REGS, "register index must be 0–4");
        Instruction {
            opcode: Opcode::Imm,
            rd,
            rs1: 0,
            rs2: 0,
            imm: value,
        }
    }

    pub fn add(rd: u8, rs1: u8, rs2: u8) -> Self {
        assert!((rd as usize) < NUM_REGS && (rs1 as usize) < NUM_REGS && (rs2 as usize) < NUM_REGS);
        Instruction {
            opcode: Opcode::Add,
            rd,
            rs1,
            rs2,
            imm: 0,
        }
    }

    pub fn mul(rd: u8, rs1: u8, rs2: u8) -> Self {
        assert!((rd as usize) < NUM_REGS && (rs1 as usize) < NUM_REGS && (rs2 as usize) < NUM_REGS);
        Instruction {
            opcode: Opcode::Mul,
            rd,
            rs1,
            rs2,
            imm: 0,
        }
    }

    pub fn sub(rd: u8, rs1: u8, rs2: u8) -> Self {
        assert!((rd as usize) < NUM_REGS && (rs1 as usize) < NUM_REGS && (rs2 as usize) < NUM_REGS);
        Instruction {
            opcode: Opcode::Sub,
            rd,
            rs1,
            rs2,
            imm: 0,
        }
    }

    pub fn halt() -> Self {
        Instruction {
            opcode: Opcode::Halt,
            rd: 0,
            rs1: 0,
            rs2: 0,
            imm: 0,
        }
    }

    /// rd = rs1 + imm
    pub fn addi(rd: u8, rs1: u8, imm: u64) -> Self {
        assert!((rd as usize) < NUM_REGS && (rs1 as usize) < NUM_REGS);
        Instruction {
            opcode: Opcode::Addi,
            rd,
            rs1,
            rs2: 0,
            imm,
        }
    }

    pub fn assert_eq(rs1: u8, rs2: u8) -> Self {
        assert!((rs1 as usize) < NUM_REGS && (rs2 as usize) < NUM_REGS);
        Instruction {
            opcode: Opcode::AssertEq,
            rd: 0,
            rs1,
            rs2,
            imm: 0,
        }
    }
}
