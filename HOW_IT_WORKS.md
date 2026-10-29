# MM2 — How It Works (Nothing Is Simulated)

Everything in MM2 is **cryptographically real**. No mocking, no hardcoded values, no simulation. The same Rust code that passes 68 unit tests runs in your browser via WebAssembly.

---

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                        YOUR BROWSER                          │
│                                                              │
│  ┌─────────────┐     calls     ┌──────────────────────────┐ │
│  │  Next.js UI  │ ──────────▶  │  mm2.wasm (207 KB)       │ │
│  │  TypeScript   │  ◀──────────  │  Real Rust compiled to   │ │
│  │  React        │   JSON       │  WebAssembly              │ │
│  └─────────────┘              │                            │ │
│                                │  • field.rs    (Fp arith)  │ │
│                                │  • vm.rs       (execution) │ │
│                                │  • air.rs      (49 constr) │ │
│                                │  • prover.rs   (STARK)     │ │
│                                │  • verifier.rs (checks)    │ │
│                                │  • fri.rs      (low-deg)   │ │
│                                │  • merkle.rs   (SHA-256)   │ │
│                                │  • channel.rs  (Fiat-Shamir│ │
│                                │  • polynomial.rs           │ │
│                                │  • domain.rs               │ │
│                                └──────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

**There is no server.** All computation happens client-side in the WASM binary.

---

## What Happens When You Click Each Button

### 1. Execute → `vm::execute()`

The real virtual machine runs your program instruction-by-instruction:

- Initializes 5 registers to zero
- Steps through each instruction (IMM, ADD, MUL, SUB, ADDI, ASSERT_EQ, HALT)
- Records every state change into an **execution trace** (46 columns per row)
- Each row captures: clock cycle, program counter, opcode, register file, read values, write value, one-hot flags
- The trace is padded to the next power of 2

**This is identical to `cargo run` — same code, same output.**

### 2. Prove → `prover::prove()`

The real STARK prover runs the full cryptographic pipeline:

1. **Interpolation**: Each of the 46 trace columns is interpolated into a polynomial over the trace domain using Lagrange interpolation over the Goldilocks field (p = 2⁶⁴ − 2³² + 1)

2. **Low-Degree Extension (LDE)**: Polynomials are evaluated on an 8× larger coset domain

3. **Merkle Commitment**: All LDE evaluations are committed via a SHA-256 Merkle tree. The **trace root** displayed in the UI is the real root hash

4. **Fiat-Shamir**: A SHA-256-based transcript absorbs the program and trace root, then squeezes the composition challenge α — making the proof non-interactive

5. **Composition Polynomial**: All 49 constraints (40 local + 9 transition) are combined into a single polynomial C(x) using powers of α

6. **Quotient**: C(x) is divided by the vanishing polynomial Z_H(x) = x^n − 1 to produce Q(x). If all constraints hold, Q(x) is a polynomial (no remainder)

7. **FRI**: The Fast Reed-Solomon IOP proves Q(x) is low-degree by repeatedly folding with random challenges. 30 queries provide ~90 bits of soundness

8. **Query Opening**: Trace and quotient values are opened at query positions with Merkle proofs

**The proof data you can copy is a real serialized STARK proof (~100-500 KB of JSON containing Merkle proofs, FRI layers, and query responses).**

### 3. Verify → `verifier::verify()`

The real verifier checks the proof without re-executing the program:

1. **Re-derive Fiat-Shamir challenges** from the proof's public data (program + Merkle roots)
2. **Derive query indices** — the verifier computes its own random query points, it does NOT trust the prover's
3. **Check FRI**: Verify all folding layers are consistent and the final layer is constant
4. **Verify Merkle proofs**: Check that opened trace and quotient values belong to the committed trees
5. **Check quotient equation**: Confirm C(x) = Q(x) · Z_H(x) at every query point
6. **HALT check**: Verify the program ends with a HALT instruction

**If any single check fails, the proof is rejected.** Try the "False Claim" demo — the ASSERT_EQ constraint violation causes the verifier to reject.

---

## Proof That It's Real

### Test Suite (68 tests, all pass)

```
$ cargo test
running 68 tests

test field::tests::test_basic_arithmetic ... ok
test field::tests::test_inverse ... ok
test field::tests::test_root_of_unity ... ok
test vm::tests::test_simple_program ... ok
test vm::tests::test_addi ... ok
test vm::tests::test_failed_assert_eq_stops_the_honest_vm ... ok
test air::tests::test_constraints_hold_for_valid_trace ... ok
test air::tests::test_constraints_catch_tampered_trace ... ok
test air::tests::test_failed_assertion_violates_constraints ... ok
test polynomial::tests::test_interpolation ... ok
test polynomial::tests::test_vanishing ... ok
test merkle::tests::test_merkle_prove_verify ... ok
test fri::tests::test_fri_honest_polynomial ... ok
test fri::tests::test_fri_rejects_random_values ... ok
test fri::tests::test_fri_rejects_polynomial_above_degree_bound ... ok
test verifier::tests::test_valid_proof_verifies ... ok
test verifier::tests::test_tampered_trace_root_fails ... ok
test verifier::tests::test_proof_of_tampered_register_is_rejected ... ok
test verifier::tests::test_proof_of_wrong_arithmetic_is_rejected ... ok
test verifier::tests::test_proof_of_a_false_assertion_is_rejected ... ok
test programs::tests::test_cubes_claim_has_a_valid_proof ... ok
test programs::tests::test_squares_claim_has_no_valid_proof ... ok
... (68 total)

test result: ok. 68 passed; 0 failed
```

### Soundness Tests (Proofs of invalid traces ARE rejected)

These tests prove the system isn't just rubber-stamping everything:

| Test | What it does | Result |
|------|-------------|--------|
| `test_proof_of_tampered_register_is_rejected` | Tampers r2 in the trace | ❌ REJECTED |
| `test_proof_of_wrong_arithmetic_is_rejected` | Claims ADD wrote 8 instead of 7 | ❌ REJECTED |
| `test_proof_with_out_of_range_opcode_is_rejected` | Sets opcode to 17 | ❌ REJECTED |
| `test_proof_with_tampered_quotient_is_rejected` | Modifies a quotient value | ❌ REJECTED |
| `test_proof_of_a_false_assertion_is_rejected` | 3²+4² ≠ 6² | ❌ REJECTED |
| `test_fri_rejects_random_values` | Random data instead of polynomial | ❌ REJECTED |
| `test_fri_rejects_polynomial_above_degree_bound` | Degree 40 > bound 8 | ❌ REJECTED |

### WASM Build Proof

The WASM binary is compiled directly from the Rust source:

```
$ wasm-pack build --target web --out-dir frontend/pkg
[INFO]: Compiling to Wasm...
   Compiling mm2 v0.1.0
    Finished release [optimized] in 19.51s
[INFO]: Your wasm pkg is ready at frontend/pkg
```

Output: `mm2_bg.wasm` (207 KB) — this IS the Rust code, compiled.

---

## What Makes This Different From a Simulation

| Aspect | Simulated | MM2 (Real) |
|--------|-----------|------------|
| Field arithmetic | Fake numbers | Real Goldilocks field (mod p = 2⁶⁴−2³²+1) |
| Execution trace | Hardcoded table | VM actually executes, records every state |
| Merkle trees | Random hex strings | Real SHA-256 hash trees |
| FRI protocol | Skipped | Real polynomial folding with 30 queries |
| Proof data | Empty/fake JSON | 100-500 KB of real Merkle proofs + FRI data |
| Verification | Always returns true | Re-derives challenges, checks every query |
| False claims | Would "verify" | Correctly rejected (try the demo!) |
| Timing | Instant | Real computation time (varies by program size) |

---

## Known Limitations

1. **Output binding gap**: The claimed register outputs are NOT cryptographically bound to the committed trace. A malicious prover could claim different outputs. This is documented in the code and tests (`test_outputs_are_not_bound_known_gap`).

2. **Security level**: 30 FRI queries with 8× blowup provide ~90 bits of security. Production systems typically target 128 bits.

3. **Performance**: The naive polynomial interpolation is O(n²). Production STARKs use NTT/FFT for O(n log n).

4. **No recursion**: This is a single-layer proof system with no proof composition or recursion.

---

## File Map

```
src/
├── field.rs        # Goldilocks field Fp (p = 2^64 - 2^32 + 1)
├── instruction.rs  # 8-opcode ISA, 5 registers
├── trace.rs        # Execution trace (46 columns per row)
├── vm.rs           # Virtual machine execution
├── air.rs          # 49 algebraic constraints (degree ≤ 3)
├── polynomial.rs   # Lagrange interpolation, Horner evaluation
├── domain.rs       # Trace domain + LDE coset domain (8× blowup)
├── merkle.rs       # SHA-256 Merkle tree commitments
├── channel.rs      # Fiat-Shamir transcript (SHA-256 sponge)
├── fri.rs          # FRI low-degree testing (30 queries)
├── prover.rs       # STARK prover (full pipeline)
├── verifier.rs     # STARK verifier (no re-execution)
├── programs.rs     # Demo programs (Nicomachus's theorem)
├── wasm.rs         # WebAssembly bindings (7 exported functions)
├── lib.rs          # Module declarations
└── main.rs         # CLI demo

frontend/
├── src/app/page.tsx           # Landing page
├── src/app/explorer/page.tsx  # Interactive STARK explorer
├── src/lib/wasm.ts            # WASM bridge (TypeScript)
├── pkg/mm2_bg.wasm            # Compiled WASM (207 KB)
└── public/mm2_bg.wasm         # Served to browser
```
