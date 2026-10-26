/// Example programs for the demo and the tests.

use crate::instruction::Instruction;

/// A program that checks the claim
///
///   1^power + 2^power + ... + n^power  ==  (1 + 2 + ... + n)^2
///
/// This is Nicomachus's theorem for `power = 3`, so the claim is **true** for cubes
/// (for n = 4 both sides are 100). For `power = 2` the claim is **false**
/// (for n = 4: 30 against 100).
///
/// Registers:
///   r0 = k, the counter                 r3 = k^power (scratch)
///   r1 = running sum of k^power         r4 = (1 + ... + n)^2 at the end
///   r2 = running sum 1 + ... + k
///
/// Length: 3 + n * (3 + power) + 3 instructions (26 for n = 4, power = 3).
pub fn power_sum_claim(n: u64, power: u32) -> Vec<Instruction> {
    assert!(power == 2 || power == 3, "only power 2 and 3 are generated");
    let mut p = vec![
        Instruction::imm(0, 0), // r0 = k = 0
        Instruction::imm(1, 0), // r1 = 0
        Instruction::imm(2, 0), // r2 = 0
    ];
    for _ in 0..n {
        p.push(Instruction::addi(0, 0, 1)); // k = k + 1
        p.push(Instruction::mul(3, 0, 0)); // r3 = k^2
        if power == 3 {
            p.push(Instruction::mul(3, 3, 0)); // r3 = k^3
        }
        p.push(Instruction::add(1, 1, 3)); // r1 += k^power
        p.push(Instruction::add(2, 2, 0)); // r2 += k
    }
    p.push(Instruction::mul(4, 2, 2)); // r4 = (1 + ... + n)^2
    p.push(Instruction::assert_eq(1, 4)); // require the two sides to be equal
    p.push(Instruction::halt());
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{air, prover, verifier, vm};

    #[test]
    fn test_program_length() {
        assert_eq!(power_sum_claim(4, 3).len(), 26);
        assert_eq!(power_sum_claim(4, 2).len(), 22);
    }

    #[test]
    fn test_cubes_claim_runs_and_gives_100() {
        let trace = vm::execute(&power_sum_claim(4, 3));
        assert_eq!(trace.real_len, 26);
        // The last row is HALT; its registers are the state after ASSERT_EQ.
        let regs = &trace.rows.last().unwrap().regs;
        assert_eq!(regs[0].value(), 4); // k
        assert_eq!(regs[1].value(), 100); // 1 + 8 + 27 + 64
        assert_eq!(regs[2].value(), 10); // 1 + 2 + 3 + 4
        assert_eq!(regs[4].value(), 100); // 10^2
    }

    #[test]
    fn test_cubes_claim_holds_for_many_n() {
        // The VM panics if the assertion fails, so this checks the identity for n = 1..=8.
        for n in 1..=8 {
            vm::execute(&power_sum_claim(n, 3));
        }
    }

    #[test]
    #[should_panic(expected = "ASSERT_EQ failed")]
    fn test_squares_claim_is_refused_by_the_honest_vm() {
        vm::execute(&power_sum_claim(4, 2));
    }

    #[test]
    fn test_cubes_claim_has_a_valid_proof() {
        let program = power_sum_claim(4, 3);
        let mut trace = vm::execute(&program);
        trace.pad_to_power_of_two();
        assert_eq!(trace.len(), 32);
        assert!(air::verify_trace_constraints(&trace.rows).is_ok());
        let proof = prover::prove(&trace, &program);
        assert!(verifier::verify(&proof).is_ok(), "{:?}", verifier::verify(&proof).err());
    }

    #[test]
    fn test_squares_claim_has_no_valid_proof() {
        let program = power_sum_claim(4, 2);
        // A cheating prover runs the VM past the failed assertion.
        let mut trace = vm::execute_unchecked(&program);
        trace.pad_to_power_of_two();
        // The constraints fail exactly on the ASSERT_EQ row (second to last instruction).
        let (row, _) = air::verify_trace_constraints(&trace.rows).unwrap_err();
        assert_eq!(row, program.len() - 2);
        let proof = prover::prove(&trace, &program);
        assert!(verifier::verify(&proof).is_err());
    }
}
