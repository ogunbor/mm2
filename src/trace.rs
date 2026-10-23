use crate::field::Fp;
use crate::instruction::{Instruction, Opcode};

pub use crate::instruction::NUM_REGS;

/// Number of opcodes in the ISA.
pub const NUM_OPCODES: usize = 8;

pub const COL_CLK: usize = 0;
pub const COL_PC: usize = 1;
pub const COL_OPCODE: usize = 2;
pub const COL_RD: usize = 3;
pub const COL_RS1: usize = 4;
pub const COL_RS2: usize = 5;
pub const COL_IMM: usize = 6;
pub const COL_REG0: usize = 7;
pub const COL_RS1_VAL: usize = COL_REG0 + NUM_REGS;
pub const COL_RS2_VAL: usize = COL_RS1_VAL + 1;
pub const COL_WRITE_VAL: usize = COL_RS2_VAL + 1;
pub const COL_OP_FLAG0: usize = COL_WRITE_VAL + 1;
pub const COL_RD_FLAG0: usize = COL_OP_FLAG0 + NUM_OPCODES;
pub const COL_RS1_FLAG0: usize = COL_RD_FLAG0 + NUM_REGS;
pub const COL_RS2_FLAG0: usize = COL_RS1_FLAG0 + NUM_REGS;

pub const NUM_COLUMNS: usize = COL_RS2_FLAG0 + NUM_REGS;

#[derive(Debug, Clone, Copy)]
pub struct TraceRow {
    pub clk: Fp,
    pub pc: Fp,
    pub opcode: Fp,
    pub rd: Fp,
    pub rs1: Fp,
    pub rs2: Fp,
    pub imm: Fp,
    /// Register state before this instruction executes.
    pub regs: [Fp; NUM_REGS],
    pub rs1_val: Fp,
    pub rs2_val: Fp,
    /// Value written to `rd` by this instruction (0 for NOP and HALT).
    pub write_val: Fp,
}

impl TraceRow {
    pub fn new(clk: u64, pc: u64, inst: &Instruction, regs: &[Fp; NUM_REGS]) -> Self {
        let rs1_val = regs[inst.rs1 as usize];
        let rs2_val = regs[inst.rs2 as usize];
        let write_val = match inst.opcode {
            Opcode::Nop | Opcode::Halt | Opcode::AssertEq => Fp::ZERO,
            Opcode::Addi => rs1_val + Fp::new(inst.imm),
            Opcode::Imm => Fp::new(inst.imm),
            Opcode::Add => rs1_val + rs2_val,
            Opcode::Mul => rs1_val * rs2_val,
            Opcode::Sub => rs1_val - rs2_val,
        };
        TraceRow {
            clk: Fp::new(clk),
            pc: Fp::new(pc),
            opcode: Fp::new(inst.opcode as u64),
            rd: Fp::new(inst.rd as u64),
            rs1: Fp::new(inst.rs1 as u64),
            rs2: Fp::new(inst.rs2 as u64),
            imm: Fp::new(inst.imm),
            regs: *regs,
            rs1_val,
            rs2_val,
            write_val,
        }
    }

    /// Convert this row into an array of field elements for the AIR.
    pub fn to_array(&self) -> [Fp; NUM_COLUMNS] {
        let mut a = [Fp::ZERO; NUM_COLUMNS];
        a[COL_CLK] = self.clk;
        a[COL_PC] = self.pc;
        a[COL_OPCODE] = self.opcode;
        a[COL_RD] = self.rd;
        a[COL_RS1] = self.rs1;
        a[COL_RS2] = self.rs2;
        a[COL_IMM] = self.imm;
        for i in 0..NUM_REGS {
            a[COL_REG0 + i] = self.regs[i];
        }
        a[COL_RS1_VAL] = self.rs1_val;
        a[COL_RS2_VAL] = self.rs2_val;
        a[COL_WRITE_VAL] = self.write_val;

        let flag = |v: Fp, k: usize| {
            if v == Fp::new(k as u64) {
                Fp::ONE
            } else {
                Fp::ZERO
            }
        };
        for k in 0..NUM_OPCODES {
            a[COL_OP_FLAG0 + k] = flag(self.opcode, k);
        }
        for i in 0..NUM_REGS {
            a[COL_RD_FLAG0 + i] = flag(self.rd, i);
            a[COL_RS1_FLAG0 + i] = flag(self.rs1, i);
            a[COL_RS2_FLAG0 + i] = flag(self.rs2, i);
        }
        a
    }

    /// Create a NOP padding row that copies the state from the previous row.
    pub fn padding(clk: u64, prev: &TraceRow) -> Self {
        // NOP with rd = rs1 = rs2 = 0 and the register state frozen.
        TraceRow::new(clk, prev.pc.value(), &Instruction::nop(), &prev.regs)
    }
}

/// The complete execution trace, including padding rows.
#[derive(Debug, Clone)]
pub struct ExecutionTrace {
    pub rows: Vec<TraceRow>,
    pub real_len: usize,
}

impl ExecutionTrace {
    pub fn new(rows: Vec<TraceRow>) -> Self {
        let real_len = rows.len();
        ExecutionTrace { rows, real_len }
    }

    pub fn pad_to_power_of_two(&mut self) {
        let target = self.rows.len().next_power_of_two().max(2);
        while self.rows.len() < target {
            let clk = self.rows.len() as u64;
            let prev = *self.rows.last().unwrap();
            self.rows.push(TraceRow::padding(clk, &prev));
        }
    }

    /// Extract a single column as a vector of field elements.
    pub fn get_column(&self, col: usize) -> Vec<Fp> {
        self.rows.iter().map(|row| row.to_array()[col]).collect()
    }

    /// Total number of rows (including padding).
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Print the trace as a table
    pub fn print(&self) {
        let mut header = format!(
            "{:>4} | {:>4} | {:>4} | {:>3} | {:>3} | {:>3} | {:>6}",
            "clk", "pc", "op", "rd", "rs1", "rs2", "imm"
        );
        for i in 0..NUM_REGS {
            header.push_str(&format!(" | {:>6}", format!("r{}", i)));
        }
        header.push_str(&format!(" | {:>6} | {:>6} | {:>6}", "rs1v", "rs2v", "wval"));
        println!("{}", header);
        println!("{}", "-".repeat(header.len()));
        for (i, row) in self.rows.iter().enumerate() {
            let mut line = format!(
                "{:>4} | {:>4} | {:>4} | {:>3} | {:>3} | {:>3} | {:>6}",
                row.clk.value(),
                row.pc.value(),
                row.opcode.value(),
                row.rd.value(),
                row.rs1.value(),
                row.rs2.value(),
                row.imm.value(),
            );
            for r in &row.regs {
                line.push_str(&format!(" | {:>6}", r.value()));
            }
            line.push_str(&format!(
                " | {:>6} | {:>6} | {:>6}",
                row.rs1_val.value(),
                row.rs2_val.value(),
                row.write_val.value()
            ));
            if i >= self.real_len {
                line.push_str(" (pad)");
            }
            println!("{}", line);
        }
    }
}
