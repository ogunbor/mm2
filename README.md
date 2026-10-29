# MM2 — STARK Proof System

A complete, from-scratch **STARK (Scalable Transparent ARgument of Knowledge)** proof system written in Rust, with an interactive web frontend.

## What This Is

MM2 includes:

1. A **custom VM** with an 8-instruction ISA
2. An **AIR (Algebraic Intermediate Representation)** constraint system
3. A **STARK prover** that generates cryptographic proofs
4. A **STARK verifier** that checks proofs without re-executing the program
5. Supporting crypto infrastructure: **FRI**, **Merkle trees**, **Fiat-Shamir**, **Goldilocks field arithmetic**

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  Write Program  →  Execute on VM  →  Produce Trace  │
│       ↓                                    ↓        │
│  AIR Constraints (verify trace is valid)            │
│       ↓                                             │
│  STARK Prover (generate proof)                      │
│       ↓                                             │
│  STARK Verifier (verify without re-execution)       │
└─────────────────────────────────────────────────────┘
```

## Field

**Goldilocks field**: `F_p` where `p = 2^64 - 2^32 + 1`

## ISA — 8 Instructions, 7 Registers

| Opcode      | Code | Description            |
| ----------- | ---- | ---------------------- |
| `NOP`       | 0    | No-op (padding only)   |
| `IMM`       | 1    | `rd = immediate`       |
| `ADD`       | 2    | `rd = rs1 + rs2`       |
| `MUL`       | 3    | `rd = rs1 * rs2`       |
| `SUB`       | 4    | `rd = rs1 - rs2`       |
| `HALT`      | 5    | Stop execution         |
| `ADDI`      | 6    | `rd = rs1 + immediate` |
| `ASSERT_EQ` | 7    | Require `rs1 == rs2`   |

## Modules

| Module           | Description                                     |
| ---------------- | ----------------------------------------------- |
| `field.rs`       | Goldilocks field arithmetic (`Fp`)              |
| `instruction.rs` | ISA definition (opcodes, instruction struct)    |
| `trace.rs`       | Execution trace (46 columns per row)            |
| `vm.rs`          | Virtual machine execution                       |
| `air.rs`         | 49 algebraic constraints (degree ≤ 3)           |
| `polynomial.rs`  | Polynomial interpolation, evaluation, division  |
| `domain.rs`      | Trace and LDE evaluation domains                |
| `merkle.rs`      | SHA-256 Merkle tree commitments                 |
| `channel.rs`     | Fiat-Shamir transcript (non-interactive proofs) |
| `fri.rs`         | FRI low-degree testing (30 queries)             |
| `prover.rs`      | STARK prover (8x blowup, Merkle + FRI)          |
| `verifier.rs`    | STARK verifier (no re-execution needed)         |
| `programs.rs`    | Demo programs (Nicomachus's theorem)            |

## Running

```bash
cargo run
```

This proves **Nicomachus's theorem** for n=4: `1³ + 2³ + 3³ + 4³ = (1 + 2 + 3 + 4)²` (both sides = 100), and also demonstrates that a false claim (with squares instead of cubes) is correctly rejected.

## Tests

```bash
cargo test
```

## Frontend

The web frontend is built with Next.js + TypeScript + Tailwind CSS, communicating with the Rust prover/verifier via WebAssembly.

```bash
cd frontend
npm install
npm run dev
```

See `frontend/README.md` for details.

### Contributors

- Chidubemkingsley
- Olugbenga Ayoola
- Jerry
