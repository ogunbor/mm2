/// The STARK verifier: checks a proof WITHOUT re-executing the program.

use crate::field::Fp;
use crate::trace::NUM_COLUMNS;
use crate::air;
use crate::domain::Domain;
use crate::merkle::MerkleTree;
use crate::channel::Channel;
use crate::fri;
use crate::prover::{StarkProof, BLOWUP_FACTOR, FRI_FINAL_SIZE};
use crate::instruction::Opcode;

/// Verify a STARK proof. Returns Ok(()) if valid, Err with reason if not.
pub fn verify(proof: &StarkProof) -> Result<(), String> {
    let n = proof.trace_length;
    if n < 2 || !n.is_power_of_two() {
        return Err(format!("invalid trace length {}", n));
    }
    let lde_size = n * BLOWUP_FACTOR;
    let lde_domain = Domain::lde_domain(n, BLOWUP_FACTOR);
    let lde_elements = lde_domain.elements();
    // omega^(n-1): the last row of the trace domain (transition constraints skip it).
    let last_row_point = Domain::trace_domain(n).generator.pow(n as u64 - 1);

    println!("  Verifying proof (trace_length={}, lde_size={})", n, lde_size);

    // Step 1: Re-derive Fiat-Shamir challenges 
    let mut channel = Channel::new();
    // Absorb the program (public input) so challenges are bound to it.
    // Without this, a valid proof for program A could be resubmitted as a proof for program B.
    for instr in &proof.program {
        channel.absorb(&[instr.opcode as u8, instr.rd, instr.rs1, instr.rs2]);
        channel.absorb(&instr.imm.to_le_bytes());
    }
    channel.absorb(&proof.trace_root);
    let alpha = channel.squeeze_field();
    channel.absorb(&proof.quotient_root);

    println!("  Re-derived alpha: {}", alpha);

    // Step 2: Replay FRI commit phase (absorb layer roots, derive betas)
    // This advances the channel through the same transcript as the prover's fri_commit.
    let fri_betas = fri::fri_derive_betas(&proof.fri_proof, &mut channel);

    // Step 3: Derive query indices
    // The verifier MUST derive its own query indices from the Fiat-Shamir transcript,
    // not trust the ones in the proof. Otherwise a malicious prover could choose
    // favorable query points or supply zero queries.
    let max_queries = (lde_size / 4).min(fri::NUM_FRI_QUERIES);
    let query_indices = channel.squeeze_indices(max_queries, lde_size / 2);

    // Step 4: Verify FRI 
    // The first FRI layer must be the committed quotient itself. Without this link a
    // prover could run FRI on some unrelated low-degree polynomial and open a different
    // (invalid) quotient for the constraint check.
    if proof.fri_proof.layer_roots.first() != Some(&proof.quotient_root) {
        return Err("FRI first layer is not the committed quotient".to_string());
    }

    // FRI checks the quotient has degree < QUOTIENT_DEGREE_FACTOR * n: the folded final
    // layer must be constant.
    fri::fri_verify_queries(
        &proof.fri_proof,
        &fri_betas,
        lde_domain.generator,
        lde_domain.offset,
        lde_size,
        FRI_FINAL_SIZE,
        &query_indices,
    ).map_err(|e| format!("FRI verification failed: {}", e))?;

    println!("  FRI verification passed");

    if proof.query_responses.len() != query_indices.len() {
        return Err(format!(
            "expected {} query responses, got {}",
            query_indices.len(), proof.query_responses.len()
        ));
    }

    for (q_idx, qr) in proof.query_responses.iter().enumerate() {
        // Use the verifier-derived index, not the prover-supplied one.
        let idx = query_indices[q_idx];
        let next_idx = (idx + BLOWUP_FACTOR) % lde_size;

        // 3a. Verify trace Merkle proofs
        let trace_leaf = encode_trace_values(&qr.trace_values);
        if !MerkleTree::verify(&proof.trace_root, idx, &trace_leaf, &qr.trace_proof, lde_size) {
            return Err(format!("Trace Merkle proof failed at query {}, index {}", q_idx, idx));
        }

        let trace_next_leaf = encode_trace_values(&qr.trace_next_values);
        if !MerkleTree::verify(&proof.trace_root, next_idx, &trace_next_leaf, &qr.trace_next_proof, lde_size) {
            return Err(format!("Trace next Merkle proof failed at query {}, index {}", q_idx, next_idx));
        }

        // 3b. Verify quotient Merkle proof
        let quotient_leaf = qr.quotient_value.value().to_le_bytes().to_vec();
        if !MerkleTree::verify(&proof.quotient_root, idx, &quotient_leaf, &qr.quotient_proof, lde_size) {
            return Err(format!("Quotient Merkle proof failed at query {}", q_idx));
        }

        // The quotient opened here must be the same value FRI opened in its first layer.
        // (idx < lde_size / 2, so it is the "lo" position of FRI layer 0.)
        if proof.fri_proof.query_proofs[q_idx].layers[0].value != qr.quotient_value {
            return Err(format!("quotient opening disagrees with FRI layer 0 at query {}", q_idx));
        }

        // 3c. Re-evaluate composition polynomial at this point
        let x = lde_elements[idx];
        let last_row_factor = x - last_row_point;
        let composition_value = air::evaluate_composition(
            &qr.trace_values,
            &qr.trace_next_values,
            alpha,
            last_row_factor,
        );

        // 3d. Compute Z_H(x) at this LDE point
        let z_h = x.pow(n as u64) - Fp::ONE;

        // 3e. Check: C(x) == Q(x) * Z_H(x)
        let expected = qr.quotient_value * z_h;
        if composition_value != expected {
            return Err(format!(
                "Quotient check failed at query {}: C(x)={:?} != Q(x)*Z_H(x)={:?}",
                q_idx, composition_value, expected
            ));
        }
    }

    println!("  All {} query checks passed", proof.query_responses.len());

    // Step 4: Minimal boundary check
    // We only check that the declared program ends with HALT.
    // The claimed outputs are NOT verified against the committed trace.
    // In a production system, boundary constraints would be folded into
    // the composition polynomial to cryptographically bind outputs.

    if proof.program.is_empty() || proof.real_trace_length == 0 || proof.real_trace_length > proof.program.len() {
        return Err("invalid program or real_trace_length".to_string());
    }
    let last_instr = &proof.program[proof.real_trace_length - 1];
    if last_instr.opcode != Opcode::Halt {
        return Err("program does not end with HALT".to_string());
    }

    println!("  HALT check passed (outputs are unverified claims)");
    println!("  Claimed (unverified) outputs: {:?}",
        proof.outputs.iter().map(|v| v.value()).collect::<Vec<_>>());

    Ok(())
}

/// Encode trace values into bytes for Merkle leaf hashing.
fn encode_trace_values(values: &[Fp; NUM_COLUMNS]) -> Vec<u8> {
    let mut data = Vec::with_capacity(NUM_COLUMNS * 8);
    for v in values {
        data.extend_from_slice(&v.value().to_le_bytes());
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::Instruction;
    use crate::vm;
    use crate::prover;

    #[test]
    fn test_valid_proof_verifies() {
        let program = vec![
            Instruction::imm(0, 3),
            Instruction::imm(1, 4),
            Instruction::add(2, 0, 1),
            Instruction::mul(3, 2, 2),
            Instruction::sub(0, 3, 2),
            Instruction::halt(),
        ];

        let mut trace = vm::execute(&program);
        trace.pad_to_power_of_two();

        let proof = prover::prove(&trace, &program);
        let result = verify(&proof);
        assert!(result.is_ok(), "Valid proof should verify: {:?}", result.err());
    }

    #[test]
    fn test_outputs_are_not_bound_known_gap() {
        let program = vec![
            Instruction::imm(0, 3),
            Instruction::imm(1, 4),
            Instruction::add(2, 0, 1),
            Instruction::halt(),
        ];

        let mut trace = vm::execute(&program);
        trace.pad_to_power_of_two();

        let mut proof = prover::prove(&trace, &program);
        // Tamper: claim r2 = 999 instead of 7
        proof.outputs[2] = Fp::new(999);

        // KNOWN GAP: outputs are not bound to the committed trace, so the forged claim still
        // verifies. This test documents that. Once output binding is added it must flip to
        // assert!(verify(&proof).is_err()).
        assert!(verify(&proof).is_ok());
    }

    #[test]
    fn test_tampered_trace_root_fails() {
        let program = vec![
            Instruction::imm(0, 10),
            Instruction::halt(),
        ];

        let mut trace = vm::execute(&program);
        trace.pad_to_power_of_two();

        let mut proof = prover::prove(&trace, &program);
        // Tamper with trace root — Merkle proofs will fail
        proof.trace_root[0] ^= 0xFF;

        let result = verify(&proof);
        assert!(result.is_err(), "Tampered trace root should fail");
    }

    // Soundness regression tests: these proofs must be REJECTED.
    // Before the FRI and degree fixes, every one of these was wrongly accepted.

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

    fn prove_trace(trace: &crate::trace::ExecutionTrace, program: &[Instruction]) -> StarkProof {
        prover::prove(trace, program)
    }

    #[test]
    fn test_proof_of_tampered_register_is_rejected() {
        let program = demo_program();
        let mut trace = vm::execute(&program);
        trace.pad_to_power_of_two();
        trace.rows[3].regs[2] = Fp::new(999);
        // Sanity: the direct constraint check already rejects this trace.
        assert!(air::verify_trace_constraints(&trace.rows).is_err());
        let proof = prove_trace(&trace, &program);
        assert!(verify(&proof).is_err(), "a proof of an invalid trace must not verify");
    }

    #[test]
    fn test_proof_of_wrong_arithmetic_is_rejected() {
        // The ADD row claims r2 = 8 instead of 7 (write_val and the next register state agree
        // with each other, so only the "write_val = rs1_val + rs2_val" constraint is violated).
        let program = demo_program();
        let mut trace = vm::execute(&program);
        trace.pad_to_power_of_two();
        trace.rows[2].write_val = Fp::new(8);
        for row in trace.rows.iter_mut().skip(3) {
            row.regs[2] = Fp::new(8);
        }
        let proof = prove_trace(&trace, &program);
        assert!(verify(&proof).is_err());
    }

    #[test]
    fn test_proof_with_out_of_range_opcode_is_rejected() {
        let program = demo_program();
        let mut trace = vm::execute(&program);
        trace.pad_to_power_of_two();
        trace.rows[1].opcode = Fp::new(17);
        let proof = prove_trace(&trace, &program);
        assert!(verify(&proof).is_err());
    }

    #[test]
    fn test_proof_with_tampered_quotient_is_rejected() {
        let program = demo_program();
        let mut trace = vm::execute(&program);
        trace.pad_to_power_of_two();
        let mut proof = prove_trace(&trace, &program);
        proof.query_responses[0].quotient_value = proof.query_responses[0].quotient_value + Fp::ONE;
        assert!(verify(&proof).is_err());
    }

    #[test]
    fn test_proof_with_unlinked_fri_is_rejected() {
        // FRI must run on the committed quotient, not on some other polynomial.
        let program = demo_program();
        let mut trace = vm::execute(&program);
        trace.pad_to_power_of_two();
        let mut proof = prove_trace(&trace, &program);
        proof.fri_proof.layer_roots[0][0] ^= 0xFF;
        assert!(verify(&proof).is_err());
    }

    #[test]
    fn test_valid_proofs_for_small_and_larger_traces() {
        // 2-row trace (smallest) and a 16-row trace using the high registers.
        let tiny = vec![Instruction::imm(0, 10), Instruction::halt()];
        let mut t = vm::execute(&tiny);
        t.pad_to_power_of_two();
        let p = prove_trace(&t, &tiny);
        assert!(verify(&p).is_ok(), "{:?}", verify(&p).err());

        let mut longer = vec![Instruction::imm(4, 2), Instruction::imm(5, 3)];
        for _ in 0..10 {
            longer.push(Instruction::mul(6, 4, 5));
            longer.push(Instruction::add(4, 4, 6));
        }
        longer.push(Instruction::halt());
        let mut t = vm::execute(&longer);
        t.pad_to_power_of_two();
        assert!(t.len() >= 16);
        let p = prove_trace(&t, &longer);
        assert!(verify(&p).is_ok(), "{:?}", verify(&p).err());
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
    fn test_proof_using_addi_and_assert_eq_verifies() {
        let program = pythagoras(1);
        let mut trace = vm::execute(&program);
        trace.pad_to_power_of_two();
        let proof = prove_trace(&trace, &program);
        assert!(verify(&proof).is_ok(), "{:?}", verify(&proof).err());
    }

    #[test]
    fn test_proof_of_a_false_assertion_is_rejected() {
        // 3^2 + 4^2 = 6^2 is false. A cheating prover runs the VM past the failed ASSERT_EQ.
        let program = pythagoras(2);
        let mut trace = vm::execute_unchecked(&program);
        trace.pad_to_power_of_two();
        let proof = prove_trace(&trace, &program);
        assert!(verify(&proof).is_err(), "a proof of a false assertion must not verify");
    }
}
