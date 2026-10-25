/// Algebraic Intermediate Representation (AIR) — the constraint system.
///
/// This module defines the algebraic rules that a valid execution trace must satisfy.
use crate::field::Fp;
use crate::trace::*;

pub const NUM_LOCAL_CONSTRAINTS: usize = NUM_OPCODES + 3 * NUM_REGS + 4 + 4 + 2 + 1 + 1;

/// Number of transition constraints: clock, pc, and one per register.
pub const NUM_TRANSITION_CONSTRAINTS: usize = 2 + NUM_REGS;

/// Total number of constraints.
pub const NUM_CONSTRAINTS: usize = NUM_LOCAL_CONSTRAINTS + NUM_TRANSITION_CONSTRAINTS;

/// Maximum degree of any constraint (used to size the quotient degree bound).
pub const CONSTRAINT_DEGREE: usize = 3;

/// Evaluate the local constraints on a single row. All should be zero for a valid row.
pub fn evaluate_local_constraints(row: &[Fp; NUM_COLUMNS]) -> [Fp; NUM_LOCAL_CONSTRAINTS] {
    let mut out = [Fp::ZERO; NUM_LOCAL_CONSTRAINTS];
    let mut n = 0;

    // Booleanity: every flag is 0 or 1.  f * (f - 1) = 0.   (degree 2)
    for col in COL_OP_FLAG0..NUM_COLUMNS {
        out[n] = row[col] * (row[col] - Fp::ONE);
        n += 1;
    }

    // Each flag group sums to exactly 1 (so exactly one flag is set). (degree 1)
    let mut sum = Fp::ZERO;
    for k in 0..NUM_OPCODES {
        sum = sum + row[COL_OP_FLAG0 + k];
    }
    out[n] = sum - Fp::ONE;
    n += 1;
    for base in [COL_RD_FLAG0, COL_RS1_FLAG0, COL_RS2_FLAG0] {
        let mut sum = Fp::ZERO;
        for i in 0..NUM_REGS {
            sum = sum + row[base + i];
        }
        out[n] = sum - Fp::ONE;
        n += 1;
    }

    // Flags agree with the plain columns: opcode = sum k * f_k, etc. (degree 1)
    // Together with booleanity and "sums to 1" this also range-checks the values.
    let mut opcode = Fp::ZERO;
    for k in 0..NUM_OPCODES {
        opcode = opcode + Fp::new(k as u64) * row[COL_OP_FLAG0 + k];
    }
    out[n] = row[COL_OPCODE] - opcode;
    n += 1;
    for (plain, base) in [(COL_RD, COL_RD_FLAG0), (COL_RS1, COL_RS1_FLAG0), (COL_RS2, COL_RS2_FLAG0)] {
        let mut idx = Fp::ZERO;
        for i in 0..NUM_REGS {
            idx = idx + Fp::new(i as u64) * row[base + i];
        }
        out[n] = row[plain] - idx;
        n += 1;
    }

    // Register reads: rs1_val = sum a_i * r_i, rs2_val = sum b_i * r_i.  (degree 2)
    let mut rs1_expected = Fp::ZERO;
    let mut rs2_expected = Fp::ZERO;
    for i in 0..NUM_REGS {
        rs1_expected = rs1_expected + row[COL_RS1_FLAG0 + i] * row[COL_REG0 + i];
        rs2_expected = rs2_expected + row[COL_RS2_FLAG0 + i] * row[COL_REG0 + i];
    }
    out[n] = row[COL_RS1_VAL] - rs1_expected;
    n += 1;
    out[n] = row[COL_RS2_VAL] - rs2_expected;
    n += 1;

    // write_val is what this instruction writes.  (degree 3: f_mul * rs1_val * rs2_val)
    let rs1v = row[COL_RS1_VAL];
    let rs2v = row[COL_RS2_VAL];
    let f_imm = row[COL_OP_FLAG0 + OP_IMM];
    let f_add = row[COL_OP_FLAG0 + OP_ADD];
    let f_mul = row[COL_OP_FLAG0 + OP_MUL];
    let f_sub = row[COL_OP_FLAG0 + OP_SUB];
    let f_addi = row[COL_OP_FLAG0 + OP_ADDI];
    let expected_write = f_imm * row[COL_IMM]
        + f_add * (rs1v + rs2v)
        + f_mul * (rs1v * rs2v)
        + f_sub * (rs1v - rs2v)
        + f_addi * (rs1v + row[COL_IMM]);
    out[n] = row[COL_WRITE_VAL] - expected_write;
    n += 1;

    // ASSERT_EQ: on an ASSERT_EQ row the two operands must be equal.  (degree 2)
    out[n] = row[COL_OP_FLAG0 + OP_ASSERT_EQ] * (rs1v - rs2v);
    n += 1;

    debug_assert_eq!(n, NUM_LOCAL_CONSTRAINTS);
    out
}

// Opcode numbers (must match `instruction::Opcode`).
const OP_IMM: usize = 1;
const OP_ADD: usize = 2;
const OP_MUL: usize = 3;
const OP_SUB: usize = 4;
const OP_ADDI: usize = 6;
const OP_ASSERT_EQ: usize = 7;

/// Sum of the flags of instructions that write a register (IMM, ADD, MUL, SUB, ADDI). Degree 1.
fn is_write(row: &[Fp; NUM_COLUMNS]) -> Fp {
    row[COL_OP_FLAG0 + OP_IMM]
        + row[COL_OP_FLAG0 + OP_ADD]
        + row[COL_OP_FLAG0 + OP_MUL]
        + row[COL_OP_FLAG0 + OP_SUB]
        + row[COL_OP_FLAG0 + OP_ADDI]
}

/// Evaluate the transition constraints between two adjacent rows. All should be zero
/// for every pair of consecutive real rows.
pub fn evaluate_transition_constraints(
    current: &[Fp; NUM_COLUMNS],
    next: &[Fp; NUM_COLUMNS],
) -> [Fp; NUM_TRANSITION_CONSTRAINTS] {
    let mut out = [Fp::ZERO; NUM_TRANSITION_CONSTRAINTS];
    let w = is_write(current);

    // Clock increments by 1.  (degree 1)
    out[0] = next[COL_CLK] - current[COL_CLK] - Fp::ONE;

    // PC advances by 1 for writing instructions and for ASSERT_EQ, and stays put for
    // NOP and HALT.  (degree 1)
    out[1] = next[COL_PC] - current[COL_PC] - (w + current[COL_OP_FLAG0 + OP_ASSERT_EQ]);

    // Register updates.  (degree 3)
    // next.r_i = cur.r_i + d_i * is_write * (write_val - cur.r_i)
    // If this instruction writes register i the new value is write_val, otherwise unchanged.
    for i in 0..NUM_REGS {
        let d_i = current[COL_RD_FLAG0 + i];
        let cur = current[COL_REG0 + i];
        out[2 + i] = next[COL_REG0 + i] - cur - d_i * w * (current[COL_WRITE_VAL] - cur);
    }
    out
}

/// Combine all constraints into one value with a random linear combination.

pub fn evaluate_composition(
    current: &[Fp; NUM_COLUMNS],
    next: &[Fp; NUM_COLUMNS],
    alpha: Fp,
    last_row_factor: Fp,
) -> Fp {
    let local = evaluate_local_constraints(current);
    let transition = evaluate_transition_constraints(current, next);

    let mut alpha_power = Fp::ONE;
    let mut local_sum = Fp::ZERO;
    for c in &local {
        local_sum = local_sum + alpha_power * *c;
        alpha_power = alpha_power * alpha;
    }
    let mut transition_sum = Fp::ZERO;
    for c in &transition {
        transition_sum = transition_sum + alpha_power * *c;
        alpha_power = alpha_power * alpha;
    }
    local_sum + last_row_factor * transition_sum
}

pub fn verify_trace_constraints(
    rows: &[crate::trace::TraceRow],
) -> Result<(), (usize, usize)> {
    for i in 0..rows.len() {
        let current = rows[i].to_array();
        let local = evaluate_local_constraints(&current);
        for (j, c) in local.iter().enumerate() {
            if *c != Fp::ZERO {
                return Err((i, j));
            }
        }
        if i + 1 < rows.len() {
            let next = rows[i + 1].to_array();
            let transition = evaluate_transition_constraints(&current, &next);
            for (j, c) in transition.iter().enumerate() {
                if *c != Fp::ZERO {
                    return Err((i, NUM_LOCAL_CONSTRAINTS + j));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::Instruction;
    use crate::vm;

    fn demo_program() -> Vec<Instruction> {
        vec![
            Instruction::imm(0, 3),
            Instruction::imm(1, 4),
            Instruction::add(2, 0, 1),
            Instruction::mul(3, 2, 2),
            Instruction::sub(0, 3, 2),
            Instruction::halt(),
        ]
    }

    #[test]
    fn test_opcode_numbers_match_isa() {
        use crate::instruction::Opcode;
        assert_eq!(Opcode::Imm as usize, OP_IMM);
        assert_eq!(Opcode::Add as usize, OP_ADD);
        assert_eq!(Opcode::Mul as usize, OP_MUL);
        assert_eq!(Opcode::Sub as usize, OP_SUB);
        assert_eq!(Opcode::Addi as usize, OP_ADDI);
        assert_eq!(Opcode::AssertEq as usize, OP_ASSERT_EQ);
        assert_eq!(Opcode::AssertEq as usize + 1, NUM_OPCODES);
    }

    #[test]
    fn test_constraints_hold_for_valid_trace() {
        let mut trace = vm::execute(&demo_program());
        trace.pad_to_power_of_two();
        assert!(verify_trace_constraints(&trace.rows).is_ok());
    }

    #[test]
    fn test_constraints_catch_tampered_trace() {
        let program = vec![
            Instruction::imm(0, 3),
            Instruction::imm(1, 4),
            Instruction::add(2, 0, 1),
            Instruction::halt(),
        ];
        let mut trace = vm::execute(&program);
        trace.pad_to_power_of_two();

        // Tamper: change r2 in row 3 (should be 7, set it to 999)
        trace.rows[3].regs[2] = Fp::new(999);
        assert!(verify_trace_constraints(&trace.rows).is_err());
    }

    #[test]
    fn test_high_registers_work_and_are_constrained() {
        // Uses r4, r5, r6 (only exist with 7 registers).
        let program = vec![
            Instruction::imm(4, 5),
            Instruction::imm(5, 6),
            Instruction::mul(6, 4, 5), // r6 = 30
            Instruction::add(0, 6, 4), // r0 = 35
            Instruction::halt(),
        ];
        let mut trace = vm::execute(&program);
        assert_eq!(trace.rows.last().unwrap().regs[6].value(), 30);
        trace.pad_to_power_of_two();
        assert!(verify_trace_constraints(&trace.rows).is_ok());

        trace.rows[3].regs[6] = Fp::new(31);
        assert!(verify_trace_constraints(&trace.rows).is_err());
    }

    #[test]
    fn test_out_of_range_opcode_rejected() {
        // The old design had no range checks. Now an invalid opcode zeroes every opcode
        // flag, which breaks the "flags sum to 1" constraint.
        let mut trace = vm::execute(&demo_program());
        trace.pad_to_power_of_two();
        trace.rows[2].opcode = Fp::new(17);
        assert!(verify_trace_constraints(&trace.rows).is_err());
    }

    #[test]
    fn test_out_of_range_register_rejected() {
        let mut trace = vm::execute(&demo_program());
        trace.pad_to_power_of_two();
        trace.rows[2].rd = Fp::new(9);
        assert!(verify_trace_constraints(&trace.rows).is_err());
    }

    #[test]
    fn test_wrong_write_value_rejected() {
        // Claim the ADD wrote a different value than rs1 + rs2.
        let mut trace = vm::execute(&demo_program());
        trace.pad_to_power_of_two();
        trace.rows[2].write_val = Fp::new(8);
        assert!(verify_trace_constraints(&trace.rows).is_err());
    }

    #[test]
    fn test_wrong_read_value_rejected() {
        let mut trace = vm::execute(&demo_program());
        trace.pad_to_power_of_two();
        trace.rows[3].rs1_val = Fp::new(1);
        assert!(verify_trace_constraints(&trace.rows).is_err());
    }

    fn pythagoras(step: u64) -> Vec<Instruction> {
        vec![
            Instruction::imm(0, 3),
            Instruction::imm(1, 4),
            Instruction::addi(2, 1, step),
            Instruction::mul(3, 0, 0),
            Instruction::mul(4, 1, 1),
            Instruction::add(3, 3, 4),
            Instruction::mul(5, 2, 2),
            Instruction::assert_eq(3, 5),
            Instruction::halt(),
        ]
    }

    #[test]
    fn test_addi_and_assert_eq_satisfy_constraints() {
        let mut trace = vm::execute(&pythagoras(1)); // 3^2 + 4^2 == 5^2
        trace.pad_to_power_of_two();
        assert!(verify_trace_constraints(&trace.rows).is_ok());
    }

    #[test]
    fn test_failed_assertion_violates_constraints() {
        // 3^2 + 4^2 != 6^2: only the ASSERT_EQ constraint can catch this.
        let mut trace = vm::execute_unchecked(&pythagoras(2));
        trace.pad_to_power_of_two();
        let err = verify_trace_constraints(&trace.rows).unwrap_err();
        assert_eq!(err.0, 7, "the violation is on the ASSERT_EQ row");
    }

    #[test]
    fn test_addi_wrong_write_value_rejected() {
        let mut trace = vm::execute(&pythagoras(1));
        trace.pad_to_power_of_two();
        trace.rows[2].write_val = Fp::new(6); // ADDI should have written 5
        assert!(verify_trace_constraints(&trace.rows).is_err());
    }

    #[test]
    fn test_pc_must_advance_past_assert_eq() {
        let mut trace = vm::execute(&pythagoras(1));
        trace.pad_to_power_of_two();
        // Freeze the PC after the ASSERT_EQ row (row 7), as if it behaved like NOP/HALT.
        trace.rows[8].pc = trace.rows[7].pc;
        assert!(verify_trace_constraints(&trace.rows).is_err());
    }
}
