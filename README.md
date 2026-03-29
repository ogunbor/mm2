# zkvm

An educational STARK-based ZKVM in Rust. A minimal register-based virtual machine with a complete proving pipeline: execution, trace generation, algebraic constraints (AIR), STARK proof generation, and verification. Built as a companion to the article [Under the Hood of ZK Virtual Machines](https://www.cyfrin.io/blog/under-the-hood-of-zk-virtual-machines).

## The VM

- 4 registers (`r0`–`r3`)
- 6 instructions: `NOP`, `IMM`, `ADD`, `MUL`, `SUB`, `HALT`
- No memory, no branches, no jumps
- All arithmetic in the Goldilocks field (`p = 2^64 - 2^32 + 1`)

## The Pipeline

```
Program → VM Execution → Execution Trace → Polynomial Interpolation →
Constraint Evaluation → Quotient Division → FRI Low-Degree Test → Verification
```

The prover interpolates the trace into polynomials, evaluates them on an LDE domain (8x blowup), commits via Merkle trees, computes a quotient polynomial, and runs FRI. The verifier re-derives Fiat-Shamir challenges, checks FRI, verifies Merkle proofs, and confirms `C(z) = Q(z) * Z_H(z)` at random query points. 30 queries give ~90 bits of soundness.

## Quick Start

```bash
cargo run    # Execute a program, generate and verify a STARK proof
cargo test   # Run all 37 tests
```

The demo computes `(3 + 4)^2 - (3 + 4) = 42` in six instructions and proves the execution.

## Source Files

| File | Description |
|------|-------------|
| `field.rs` | Goldilocks field arithmetic (`Fp`) |
| `polynomial.rs` | Polynomial operations, Lagrange interpolation |
| `domain.rs` | Trace and LDE evaluation domains |
| `instruction.rs` | ISA definition (6 opcodes, uniform encoding) |
| `vm.rs` | Fetch-decode-execute loop, produces execution trace |
| `trace.rs` | 13-column trace table, power-of-two padding |
| `air.rs` | 8 transition constraints, selector polynomials, composition |
| `prover.rs` | Full STARK prover |
| `verifier.rs` | STARK verifier |
| `fri.rs` | FRI protocol (commit, query, verify) |
| `merkle.rs` | Binary Merkle tree (SHA-256) |
| `channel.rs` | Fiat-Shamir transcript |

## Educational Simplifications

This is a teaching implementation. It deliberately omits features required for production use:

- **No zero-knowledge**: LDE evaluations leak trace data (no masking polynomials)
- **Boundary constraints not in composition**: claimed outputs are unverified
- **No program-consistency constraints**: the trace is not bound to the declared program
- **No range constraints**: opcode/register values are not algebraically range-checked
- **No initial-state constraints**: starting state is not verified
- **No memory, no branches**: the ISA is minimal
- **Naive O(n^2) polynomial arithmetic**: no NTT

See section 14 of the article for a full discussion of each gap and how production systems address them.

## Dependencies

The only external dependency is [`sha2`](https://crates.io/crates/sha2) for Merkle tree hashing.
