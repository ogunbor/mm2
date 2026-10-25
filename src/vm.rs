use crate::field::Fp;
use crate::instruction::{Instruction, Opcode};
use crate::trace::{ExecutionTrace, TraceRow, NUM_REGS};

/// Execute a program and return the execution trace.

pub fn execute(program: &[Instruction]) -> ExecutionTrace {
    run(program, true)
}

pub fn execute_unchecked(program: &[Instruction]) -> ExecutionTrace {
    run(program, false)
}

fn run(program: &[Instruction], check_asserts: bool) -> ExecutionTrace {
    let mut regs: [Fp; NUM_REGS] = [Fp::ZERO; NUM_REGS];
    let mut pc: usize = 0;
    let mut rows: Vec<TraceRow> = Vec::new();

    loop {
        assert!(pc < program.len(), "PC out of bounds: {}", pc);
        let inst = &program[pc];
        let clk = rows.len() as u64;

        // Record the state BEFORE executing this instruction
        let row = TraceRow::new(clk, pc as u64, inst, &regs);
        let rs1_val = row.rs1_val;
        let rs2_val = row.rs2_val;
        rows.push(row);

        // Execute the instruction
        match inst.opcode {
            // produces a trace that can never be proven. Refuse it loudly instead.
            Opcode::Nop => panic!("NOP is reserved for trace padding and cannot appear in a program"),
            Opcode::Imm => {
                regs[inst.rd as usize] = Fp::new(inst.imm);
            }
            Opcode::Add => {
                regs[inst.rd as usize] = rs1_val + rs2_val;
            }
            Opcode::Mul => {
                regs[inst.rd as usize] = rs1_val * rs2_val;
            }
            Opcode::Sub => {
                regs[inst.rd as usize] = rs1_val - rs2_val;
            }
            Opcode::Addi => {
                regs[inst.rd as usize] = rs1_val + Fp::new(inst.imm);
            }
            Opcode::AssertEq => {
                if check_asserts {
                    assert!(
                        rs1_val == rs2_val,
                        "ASSERT_EQ failed at pc {}: r{} = {} but r{} = {}",
                        pc, inst.rs1, rs1_val, inst.rs2, rs2_val
                    );
                }
            }
            Opcode::Halt => {
                break;
            }
        }

        pc += 1;
    }

    ExecutionTrace::new(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_program() {
        // (3 + 4) * (3 + 4) - 7 = 42
        let program = vec![
            Instruction::imm(0, 3),       // r0 = 3
            Instruction::imm(1, 4),       // r1 = 4
            Instruction::add(2, 0, 1),    // r2 = r0 + r1 = 7
            Instruction::mul(3, 2, 2),    // r3 = r2 * r2 = 49
            Instruction::sub(0, 3, 2),    // r0 = r3 - r2 = 42
            Instruction::halt(),
        ];
        let trace = execute(&program);
        assert_eq!(trace.real_len, 6);
        let last = &trace.rows[trace.real_len - 1];
        
        assert_eq!(last.regs[0].value(), 42);
        assert_eq!(last.regs[1].value(), 4);
        assert_eq!(last.regs[2].value(), 7);
        assert_eq!(last.regs[3].value(), 49);
    }

    #[test]
    fn test_padding() {
        let program = vec![
            Instruction::imm(0, 10),
            Instruction::halt(),
        ];
        let mut trace = execute(&program);
        assert_eq!(trace.real_len, 2);
        trace.pad_to_power_of_two();
        assert_eq!(trace.len(), 2); // already power of 2
    }

    #[test]
    fn test_padding_to_next_power() {
        let program = vec![
            Instruction::imm(0, 1),
            Instruction::imm(1, 2),
            Instruction::add(2, 0, 1),
            Instruction::halt(),
        ];
        let mut trace = execute(&program);
        assert_eq!(trace.real_len, 4);
        trace.pad_to_power_of_two();
        assert_eq!(trace.len(), 4); // 4 is already a power of 2
    }

    #[test]
    fn test_trace_columns() {
        let program = vec![
            Instruction::imm(0, 5),
            Instruction::halt(),
        ];
        let mut trace = execute(&program);
        trace.pad_to_power_of_two();

        // Column 0 is clk
        let clk_col = trace.get_column(0);
        assert_eq!(clk_col[0].value(), 0);
        assert_eq!(clk_col[1].value(), 1);
    }

    #[test]
    fn test_addi() {
        let program = vec![
            Instruction::imm(0, 5),
            Instruction::addi(1, 0, 7), // r1 = 12
            Instruction::addi(0, 0, 1), // r0 = 6 (rd == rs1 is allowed)
            Instruction::halt(),
        ];
        let trace = execute(&program);
        let last = trace.rows.last().unwrap();
        // The HALT row records the state before HALT, i.e. after the last ADDI.
        assert_eq!(last.regs[0].value(), 6);
        assert_eq!(last.regs[1].value(), 12);
    }

    #[test]
    fn test_assert_eq_passes_and_writes_nothing() {
        let program = vec![
            Instruction::imm(0, 9),
            Instruction::imm(1, 9),
            Instruction::assert_eq(0, 1),
            Instruction::halt(),
        ];
        let trace = execute(&program);
        assert_eq!(trace.real_len, 4);
        // Registers are unchanged across the ASSERT_EQ row.
        assert_eq!(trace.rows[2].regs, trace.rows[3].regs);
        // The PC advances past it.
        assert_eq!(trace.rows[3].pc.value(), 3);
    }

    #[test]
    #[should_panic(expected = "ASSERT_EQ failed")]
    fn test_failed_assert_eq_stops_the_honest_vm() {
        let program = vec![
            Instruction::imm(0, 1),
            Instruction::imm(1, 2),
            Instruction::assert_eq(0, 1),
            Instruction::halt(),
        ];
        execute(&program);
    }

    #[test]
    fn test_execute_unchecked_runs_past_a_failed_assert() {
        let program = vec![
            Instruction::imm(0, 1),
            Instruction::imm(1, 2),
            Instruction::assert_eq(0, 1),
            Instruction::halt(),
        ];
        let trace = execute_unchecked(&program);
        assert_eq!(trace.real_len, 4);
    }

    #[test]
    #[should_panic(expected = "NOP is reserved")]
    fn test_nop_in_program_is_rejected() {
        execute(&[Instruction::nop(), Instruction::halt()]);
    }
}
