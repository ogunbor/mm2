use mm2::air;
use mm2::instruction::{Instruction, Opcode};
use mm2::programs::power_sum_claim;
use mm2::prover;
use mm2::verifier;
use mm2::vm;

fn show(i: &Instruction) -> String {
    match i.opcode {
        Opcode::Nop => "NOP".to_string(),
        Opcode::Imm => format!("IMM r{}, {}", i.rd, i.imm),
        Opcode::Add => format!("ADD r{}, r{}, r{}", i.rd, i.rs1, i.rs2),
        Opcode::Mul => format!("MUL r{}, r{}, r{}", i.rd, i.rs1, i.rs2),
        Opcode::Sub => format!("SUB r{}, r{}, r{}", i.rd, i.rs1, i.rs2),
        Opcode::Addi => format!("ADDI r{}, r{}, {}", i.rd, i.rs1, i.imm),
        Opcode::AssertEq => format!("ASSERT_EQ r{}, r{}", i.rs1, i.rs2),
        Opcode::Halt => "HALT".to_string(),
    }
}

fn main() {
    println!("Field: Goldilocks (p = 2^64 - 2^32 + 1)");
    println!("ISA: 8 instructions (NOP, IMM, ADD, MUL, SUB, HALT, ADDI, ASSERT_EQ), 7 registers");
    println!("Proof system: STARK with FRI\n");

    // Step 1: Define a program
    // Nicomachus's theorem for n = 4:  1^3 + 2^3 + 3^3 + 4^3 = (1 + 2 + 3 + 4)^2.
    // Both sides are 100. The program counts with ADDI, builds each cube with MUL,
    // accumulates with ADD, and ASSERT_EQ checks the two sides at the end.
    let n = 4;
    let program = power_sum_claim(n, 3);

    println!("Program ({} instructions): prove that 1^3 + 2^3 + 3^3 + 4^3 = (1 + 2 + 3 + 4)^2", program.len());
    println!("  r0 = counter k, r1 = sum of cubes, r2 = sum of k, r3 = scratch, r4 = (sum of k)^2");
    for (pc, inst) in program.iter().enumerate() {
        println!("  {:>2}: {}", pc, show(inst));
    }
    println!();

    // Step 2: Execute and produce trace 
    println!("[1] Executing program...");
    let mut trace = vm::execute(&program);
    println!("  Execution complete. {} instructions executed (including HALT).\n", trace.real_len);

    println!("[2] Execution trace:");
    trace.print();
    println!();

    // Pad trace to power of 2
    trace.pad_to_power_of_two();
    println!("  Padded trace to {} rows (next power of 2)\n", trace.len());

    // Step 3: Verify constraints directly 
    println!("[3] Verifying AIR constraints on trace...");
    match air::verify_trace_constraints(&trace.rows) {
        Ok(()) => println!("  All constraints satisfied!\n"),
        Err((row, constraint)) => {
            println!("  CONSTRAINT VIOLATION at row {}, constraint {}", row, constraint);
            return;
        }
    }

    // Step 4: Generate STARK proof 
    println!("[4] Generating STARK proof...");
    let proof = prover::prove(&trace, &program);
    println!("  Proof generated successfully.\n");

    // Step 5: Verify the proof
    println!("[5] Verifying STARK proof (without re-executing the program)...");
    match verifier::verify(&proof) {
        Ok(()) => {
            println!("\n  *** PROOF VERIFIED ***");
            println!("  The verifier is convinced that:");
            println!("    - A committed trace satisfies all {} constraints (degree <= {})", air::NUM_CONSTRAINTS, air::CONSTRAINT_DEGREE);
            println!("    - The quotient polynomial is low-degree (FRI final layer is constant)");
            println!("  Note: the declared program is absorbed into Fiat-Shamir, but the trace");
            println!("  is not constrained to match it, so the ASSERT_EQ only counts if the trace");
            println!("  really ran this program. Outputs are unverified claims (see README).");
            println!("  Claimed (unverified) outputs: {:?}",
                proof.outputs.iter().map(|v| v.value()).collect::<Vec<_>>());
        }
        Err(e) => {
            println!("  PROOF REJECTED: {}", e);
        }
    }

    // Step 6: A false claim 
    // The same claim with squares instead of cubes is false: 30 against 100.
    println!("\n[6] A false claim: 1^2 + 2^2 + 3^2 + 4^2 = (1 + 2 + 3 + 4)^2   (30 vs 100)");
    let bad_program = power_sum_claim(n, 2);
    println!("  The honest VM refuses to run it (ASSERT_EQ fails).");
    println!("  A cheating prover forces the VM past the failed assertion instead:");
    let mut bad_trace = vm::execute_unchecked(&bad_program);
    bad_trace.pad_to_power_of_two();
    match air::verify_trace_constraints(&bad_trace.rows) {
        Ok(()) => println!("  Direct constraint check: passed (this would be a bug!)"),
        Err((row, c)) => println!("  Direct constraint check: VIOLATION at row {}, constraint {}", row, c),
    }
    let bad_proof = prover::prove(&bad_trace, &bad_program);
    match verifier::verify(&bad_proof) {
        Ok(()) => println!("  Proof of the false claim: ACCEPTED (this would be a bug!)"),
        Err(e) => println!("  Proof of the false claim: REJECTED ({})", e),
    }
}