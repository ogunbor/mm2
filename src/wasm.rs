/// WASM bindings — exposes the STARK pipeline to JavaScript.

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

use crate::instruction::{Instruction, Opcode, NUM_REGS};
use crate::vm;
use crate::air;
use crate::prover;
use crate::verifier;
use crate::field::Fp;

// ---------------------------------------------------------------------------
// Data types exposed to JS (serialized as JSON)
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone)]
pub struct JsInstruction {
    pub opcode: String,
    pub rd: u8,
    pub rs1: u8,
    pub rs2: u8,
    pub imm: u64,
}

#[derive(Serialize)]
pub struct JsTraceRow {
    pub clk: u64,
    pub pc: u64,
    pub opcode: u64,
    pub opcode_name: String,
    pub rd: u64,
    pub rs1: u64,
    pub rs2: u64,
    pub imm: u64,
    pub regs: Vec<u64>,
    pub rs1_val: u64,
    pub rs2_val: u64,
    pub write_val: u64,
    pub is_padding: bool,
}

#[derive(Serialize)]
pub struct JsExecutionResult {
    pub rows: Vec<JsTraceRow>,
    pub real_len: usize,
    pub padded_len: usize,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct JsConstraintResult {
    pub valid: bool,
    pub error_row: Option<usize>,
    pub error_constraint: Option<usize>,
}

#[derive(Serialize)]
pub struct JsProofResult {
    pub success: bool,
    pub error: Option<String>,
    pub trace_root: Option<String>,
    pub quotient_root: Option<String>,
    pub num_queries: Option<usize>,
    pub trace_length: Option<usize>,
    pub fri_layers: Option<usize>,
    pub outputs: Option<Vec<u64>>,
    /// Serialized proof for verification (base64-encoded JSON)
    pub proof_data: Option<String>,
}

#[derive(Serialize)]
pub struct JsVerifyResult {
    pub valid: bool,
    pub error: Option<String>,
    pub steps: Vec<JsVerifyStep>,
}

#[derive(Serialize)]
pub struct JsVerifyStep {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

// ---------------------------------------------------------------------------
// Conversions
// ---------------------------------------------------------------------------

fn opcode_name(op: Opcode) -> &'static str {
    match op {
        Opcode::Nop => "NOP",
        Opcode::Imm => "IMM",
        Opcode::Add => "ADD",
        Opcode::Mul => "MUL",
        Opcode::Sub => "SUB",
        Opcode::Halt => "HALT",
        Opcode::Addi => "ADDI",
        Opcode::AssertEq => "ASSERT_EQ",
    }
}

fn parse_opcode(s: &str) -> Result<Opcode, String> {
    match s.to_uppercase().as_str() {
        "NOP" => Ok(Opcode::Nop),
        "IMM" => Ok(Opcode::Imm),
        "ADD" => Ok(Opcode::Add),
        "MUL" => Ok(Opcode::Mul),
        "SUB" => Ok(Opcode::Sub),
        "HALT" => Ok(Opcode::Halt),
        "ADDI" => Ok(Opcode::Addi),
        "ASSERT_EQ" => Ok(Opcode::AssertEq),
        _ => Err(format!("Unknown opcode: {}", s)),
    }
}

fn js_to_instruction(js: &JsInstruction) -> Result<Instruction, String> {
    let opcode = parse_opcode(&js.opcode)?;
    Ok(Instruction {
        opcode,
        rd: js.rd,
        rs1: js.rs1,
        rs2: js.rs2,
        imm: js.imm,
    })
}

fn parse_program(json: &str) -> Result<Vec<Instruction>, String> {
    let js_insts: Vec<JsInstruction> =
        serde_json::from_str(json).map_err(|e| format!("JSON parse error: {}", e))?;
    js_insts.iter().map(js_to_instruction).collect()
}

fn trace_row_to_js(row: &crate::trace::TraceRow, is_padding: bool) -> JsTraceRow {
    JsTraceRow {
        clk: row.clk.value(),
        pc: row.pc.value(),
        opcode: row.opcode.value(),
        opcode_name: opcode_name(Opcode::from_u8(row.opcode.value() as u8)).to_string(),
        rd: row.rd.value(),
        rs1: row.rs1.value(),
        rs2: row.rs2.value(),
        imm: row.imm.value(),
        regs: row.regs.iter().map(|r| r.value()).collect(),
        rs1_val: row.rs1_val.value(),
        rs2_val: row.rs2_val.value(),
        write_val: row.write_val.value(),
        is_padding,
    }
}

// ---------------------------------------------------------------------------
// WASM-exported functions
// ---------------------------------------------------------------------------

/// Execute a program and return the execution trace as JSON.
#[wasm_bindgen]
pub fn execute_program(program_json: &str) -> String {
    let program = match parse_program(program_json) {
        Ok(p) => p,
        Err(e) => {
            return serde_json::to_string(&JsExecutionResult {
                rows: vec![],
                real_len: 0,
                padded_len: 0,
                success: false,
                error: Some(e),
            })
            .unwrap();
        }
    };

    // Use std::panic::catch_unwind to catch assertion failures
    let result = std::panic::catch_unwind(|| {
        let mut trace = vm::execute(&program);
        let real_len = trace.real_len;
        trace.pad_to_power_of_two();
        let padded_len = trace.len();

        let rows: Vec<JsTraceRow> = trace
            .rows
            .iter()
            .enumerate()
            .map(|(i, row)| trace_row_to_js(row, i >= real_len))
            .collect();

        (rows, real_len, padded_len)
    });

    match result {
        Ok((rows, real_len, padded_len)) => serde_json::to_string(&JsExecutionResult {
            rows,
            real_len,
            padded_len,
            success: true,
            error: None,
        })
        .unwrap(),
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "Execution failed".to_string()
            };
            serde_json::to_string(&JsExecutionResult {
                rows: vec![],
                real_len: 0,
                padded_len: 0,
                success: false,
                error: Some(msg),
            })
            .unwrap()
        }
    }
}

/// Execute a program without checking assertions.
#[wasm_bindgen]
pub fn execute_program_unchecked(program_json: &str) -> String {
    let program = match parse_program(program_json) {
        Ok(p) => p,
        Err(e) => {
            return serde_json::to_string(&JsExecutionResult {
                rows: vec![],
                real_len: 0,
                padded_len: 0,
                success: false,
                error: Some(e),
            })
            .unwrap();
        }
    };

    let mut trace = vm::execute_unchecked(&program);
    let real_len = trace.real_len;
    trace.pad_to_power_of_two();
    let padded_len = trace.len();

    let rows: Vec<JsTraceRow> = trace
        .rows
        .iter()
        .enumerate()
        .map(|(i, row)| trace_row_to_js(row, i >= real_len))
        .collect();

    serde_json::to_string(&JsExecutionResult {
        rows,
        real_len,
        padded_len,
        success: true,
        error: None,
    })
    .unwrap()
}

/// Verify AIR constraints on a trace.
#[wasm_bindgen]
pub fn check_constraints(program_json: &str) -> String {
    let program = match parse_program(program_json) {
        Ok(p) => p,
        Err(_e) => {
            return serde_json::to_string(&JsConstraintResult {
                valid: false,
                error_row: None,
                error_constraint: None,
            })
            .unwrap();
        }
    };

    let result = std::panic::catch_unwind(|| {
        let mut trace = vm::execute(&program);
        trace.pad_to_power_of_two();
        air::verify_trace_constraints(&trace.rows)
    });

    match result {
        Ok(Ok(())) => serde_json::to_string(&JsConstraintResult {
            valid: true,
            error_row: None,
            error_constraint: None,
        })
        .unwrap(),
        Ok(Err((row, constraint))) => serde_json::to_string(&JsConstraintResult {
            valid: false,
            error_row: Some(row),
            error_constraint: Some(constraint),
        })
        .unwrap(),
        Err(_) => serde_json::to_string(&JsConstraintResult {
            valid: false,
            error_row: None,
            error_constraint: None,
        })
        .unwrap(),
    }
}

/// Generate a STARK proof for a program.
#[wasm_bindgen]
pub fn generate_proof(program_json: &str) -> String {
    let program = match parse_program(program_json) {
        Ok(p) => p,
        Err(e) => {
            return serde_json::to_string(&JsProofResult {
                success: false,
                error: Some(e),
                trace_root: None,
                quotient_root: None,
                num_queries: None,
                trace_length: None,
                fri_layers: None,
                outputs: None,
                proof_data: None,
            })
            .unwrap();
        }
    };

    let result = std::panic::catch_unwind(|| {
        let mut trace = vm::execute(&program);
        trace.pad_to_power_of_two();
        prover::prove(&trace, &program)
    });

    match result {
        Ok(proof) => {
            let trace_root = hex::encode_to_string(&proof.trace_root);
            let quotient_root = hex::encode_to_string(&proof.quotient_root);
            let outputs: Vec<u64> = proof.outputs.iter().map(|v| v.value()).collect();

            // Serialize the proof for later verification
            let proof_data = serialize_proof(&proof);

            serde_json::to_string(&JsProofResult {
                success: true,
                error: None,
                trace_root: Some(trace_root),
                quotient_root: Some(quotient_root),
                num_queries: Some(proof.query_responses.len()),
                trace_length: Some(proof.trace_length),
                fri_layers: Some(proof.fri_proof.layer_roots.len()),
                outputs: Some(outputs),
                proof_data: Some(proof_data),
            })
            .unwrap()
        }
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "Proof generation failed".to_string()
            };
            serde_json::to_string(&JsProofResult {
                success: false,
                error: Some(msg),
                trace_root: None,
                quotient_root: None,
                num_queries: None,
                trace_length: None,
                fri_layers: None,
                outputs: None,
                proof_data: None,
            })
            .unwrap()
        }
    }
}

/// Verify a STARK proof.
#[wasm_bindgen]
pub fn verify_proof(proof_data: &str) -> String {
    let proof = match deserialize_proof(proof_data) {
        Ok(p) => p,
        Err(e) => {
            return serde_json::to_string(&JsVerifyResult {
                valid: false,
                error: Some(format!("Failed to deserialize proof: {}", e)),
                steps: vec![],
            })
            .unwrap();
        }
    };

    let mut steps = Vec::new();

    // Step 1: Check trace length
    let trace_ok = proof.trace_length >= 2 && proof.trace_length.is_power_of_two();
    steps.push(JsVerifyStep {
        name: "Trace Length Check".to_string(),
        passed: trace_ok,
        detail: format!("Trace length: {} (must be ≥2 and power of 2)", proof.trace_length),
    });

    // Step 2: Full verification
    match verifier::verify(&proof) {
        Ok(()) => {
            steps.push(JsVerifyStep {
                name: "Fiat-Shamir Challenges".to_string(),
                passed: true,
                detail: "Re-derived challenges match".to_string(),
            });
            steps.push(JsVerifyStep {
                name: "FRI Verification".to_string(),
                passed: true,
                detail: format!("{} FRI layers verified", proof.fri_proof.layer_roots.len()),
            });
            steps.push(JsVerifyStep {
                name: "Merkle Proofs".to_string(),
                passed: true,
                detail: format!("{} query Merkle proofs verified", proof.query_responses.len()),
            });
            steps.push(JsVerifyStep {
                name: "Quotient Check".to_string(),
                passed: true,
                detail: "C(x) = Q(x) · Z_H(x) at all query points".to_string(),
            });
            steps.push(JsVerifyStep {
                name: "HALT Check".to_string(),
                passed: true,
                detail: "Program ends with HALT".to_string(),
            });
            serde_json::to_string(&JsVerifyResult {
                valid: true,
                error: None,
                steps,
            })
            .unwrap()
        }
        Err(e) => {
            steps.push(JsVerifyStep {
                name: "Verification".to_string(),
                passed: false,
                detail: e.clone(),
            });
            serde_json::to_string(&JsVerifyResult {
                valid: false,
                error: Some(e),
                steps,
            })
            .unwrap()
        }
    }
}

/// Generate a demo program (Nicomachus's theorem).
#[wasm_bindgen]
pub fn demo_program(n: u32, power: u32) -> String {
    let program = crate::programs::power_sum_claim(n as u64, power);
    let js_insts: Vec<JsInstruction> = program
        .iter()
        .map(|inst| JsInstruction {
            opcode: opcode_name(inst.opcode).to_string(),
            rd: inst.rd,
            rs1: inst.rs1,
            rs2: inst.rs2,
            imm: inst.imm,
        })
        .collect();
    serde_json::to_string(&js_insts).unwrap()
}

/// Get ISA info (opcodes, register count, etc).
#[wasm_bindgen]
pub fn get_isa_info() -> String {
    let info = serde_json::json!({
        "field": "Goldilocks (p = 2^64 - 2^32 + 1)",
        "prime": "18446744069414584321",
        "num_registers": NUM_REGS,
        "opcodes": [
            {"code": 0, "name": "NOP", "format": "NOP", "description": "No operation (padding only)"},
            {"code": 1, "name": "IMM", "format": "IMM rd, imm", "description": "rd = immediate value"},
            {"code": 2, "name": "ADD", "format": "ADD rd, rs1, rs2", "description": "rd = rs1 + rs2"},
            {"code": 3, "name": "MUL", "format": "MUL rd, rs1, rs2", "description": "rd = rs1 * rs2"},
            {"code": 4, "name": "SUB", "format": "SUB rd, rs1, rs2", "description": "rd = rs1 - rs2"},
            {"code": 5, "name": "HALT", "format": "HALT", "description": "Stop execution"},
            {"code": 6, "name": "ADDI", "format": "ADDI rd, rs1, imm", "description": "rd = rs1 + immediate"},
            {"code": 7, "name": "ASSERT_EQ", "format": "ASSERT_EQ rs1, rs2", "description": "Require rs1 == rs2"},
        ],
        "num_trace_columns": crate::trace::NUM_COLUMNS,
        "num_constraints": air::NUM_CONSTRAINTS,
        "constraint_degree": air::CONSTRAINT_DEGREE,
        "blowup_factor": prover::BLOWUP_FACTOR,
        "num_fri_queries": crate::fri::NUM_FRI_QUERIES,
    });
    serde_json::to_string(&info).unwrap()
}

// ---------------------------------------------------------------------------
// Proof serialization (simple JSON-based, not optimized)
// ---------------------------------------------------------------------------

mod hex {
    pub fn encode_to_string(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    pub fn decode_string(s: &str) -> Result<Vec<u8>, String> {
        if s.len() % 2 != 0 {
            return Err("odd hex length".to_string());
        }
        (0..s.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| format!("hex decode: {}", e))
            })
            .collect()
    }
}

#[derive(Serialize, Deserialize)]
struct SerializedProof {
    trace_root: String,
    quotient_root: String,
    trace_length: usize,
    real_trace_length: usize,
    program: Vec<JsInstruction>,
    outputs: Vec<u64>,
    query_indices: Vec<usize>,
    fri_layer_roots: Vec<String>,
    fri_final_layer: Vec<u64>,
    query_responses: Vec<SerializedQueryResponse>,
    fri_query_proofs: Vec<SerializedFriQueryProof>,
}

#[derive(Serialize, Deserialize)]
struct SerializedQueryResponse {
    index: usize,
    trace_values: Vec<u64>,
    trace_next_values: Vec<u64>,
    trace_proof_siblings: Vec<String>,
    trace_next_proof_siblings: Vec<String>,
    quotient_value: u64,
    quotient_proof_siblings: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct SerializedFriQueryProof {
    layers: Vec<SerializedFriQueryLayer>,
}

#[derive(Serialize, Deserialize)]
struct SerializedFriQueryLayer {
    value: u64,
    sibling_value: u64,
    proof_siblings: Vec<String>,
    sibling_proof_siblings: Vec<String>,
}

fn serialize_proof(proof: &prover::StarkProof) -> String {
    let sp = SerializedProof {
        trace_root: hex::encode_to_string(&proof.trace_root),
        quotient_root: hex::encode_to_string(&proof.quotient_root),
        trace_length: proof.trace_length,
        real_trace_length: proof.real_trace_length,
        program: proof
            .program
            .iter()
            .map(|inst| JsInstruction {
                opcode: opcode_name(inst.opcode).to_string(),
                rd: inst.rd,
                rs1: inst.rs1,
                rs2: inst.rs2,
                imm: inst.imm,
            })
            .collect(),
        outputs: proof.outputs.iter().map(|v| v.value()).collect(),
        query_indices: proof.query_indices.clone(),
        fri_layer_roots: proof
            .fri_proof
            .layer_roots
            .iter()
            .map(|r| hex::encode_to_string(r))
            .collect(),
        fri_final_layer: proof.fri_proof.final_layer.iter().map(|v| v.value()).collect(),
        query_responses: proof
            .query_responses
            .iter()
            .map(|qr| SerializedQueryResponse {
                index: qr.index,
                trace_values: qr.trace_values.iter().map(|v| v.value()).collect(),
                trace_next_values: qr.trace_next_values.iter().map(|v| v.value()).collect(),
                trace_proof_siblings: qr
                    .trace_proof
                    .siblings
                    .iter()
                    .map(|s| hex::encode_to_string(s))
                    .collect(),
                trace_next_proof_siblings: qr
                    .trace_next_proof
                    .siblings
                    .iter()
                    .map(|s| hex::encode_to_string(s))
                    .collect(),
                quotient_value: qr.quotient_value.value(),
                quotient_proof_siblings: qr
                    .quotient_proof
                    .siblings
                    .iter()
                    .map(|s| hex::encode_to_string(s))
                    .collect(),
            })
            .collect(),
        fri_query_proofs: proof
            .fri_proof
            .query_proofs
            .iter()
            .map(|qp| SerializedFriQueryProof {
                layers: qp
                    .layers
                    .iter()
                    .map(|l| SerializedFriQueryLayer {
                        value: l.value.value(),
                        sibling_value: l.sibling_value.value(),
                        proof_siblings: l
                            .proof
                            .siblings
                            .iter()
                            .map(|s| hex::encode_to_string(s))
                            .collect(),
                        sibling_proof_siblings: l
                            .sibling_proof
                            .siblings
                            .iter()
                            .map(|s| hex::encode_to_string(s))
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
    };
    serde_json::to_string(&sp).unwrap()
}

fn deserialize_proof(data: &str) -> Result<prover::StarkProof, String> {
    let sp: SerializedProof =
        serde_json::from_str(data).map_err(|e| format!("JSON parse error: {}", e))?;

    let to_hash = |s: &str| -> Result<[u8; 32], String> {
        let bytes = hex::decode_string(s)?;
        let mut arr = [0u8; 32];
        if bytes.len() != 32 {
            return Err(format!("expected 32 bytes, got {}", bytes.len()));
        }
        arr.copy_from_slice(&bytes);
        Ok(arr)
    };

    let trace_root = to_hash(&sp.trace_root)?;
    let quotient_root = to_hash(&sp.quotient_root)?;

    let program: Vec<Instruction> = sp
        .program
        .iter()
        .map(js_to_instruction)
        .collect::<Result<_, _>>()?;

    let mut outputs = [Fp::ZERO; NUM_REGS];
    for (i, v) in sp.outputs.iter().enumerate() {
        if i < NUM_REGS {
            outputs[i] = Fp::new(*v);
        }
    }

    let query_responses: Vec<prover::QueryResponse> = sp
        .query_responses
        .iter()
        .map(|qr| {
            let mut trace_values = [Fp::ZERO; crate::trace::NUM_COLUMNS];
            let mut trace_next_values = [Fp::ZERO; crate::trace::NUM_COLUMNS];
            for (i, v) in qr.trace_values.iter().enumerate() {
                if i < crate::trace::NUM_COLUMNS {
                    trace_values[i] = Fp::new(*v);
                }
            }
            for (i, v) in qr.trace_next_values.iter().enumerate() {
                if i < crate::trace::NUM_COLUMNS {
                    trace_next_values[i] = Fp::new(*v);
                }
            }
            Ok(prover::QueryResponse {
                index: qr.index,
                trace_values,
                trace_next_values,
                trace_proof: crate::merkle::MerkleProof {
                    siblings: qr
                        .trace_proof_siblings
                        .iter()
                        .map(|s| to_hash(s))
                        .collect::<Result<_, _>>()?,
                },
                trace_next_proof: crate::merkle::MerkleProof {
                    siblings: qr
                        .trace_next_proof_siblings
                        .iter()
                        .map(|s| to_hash(s))
                        .collect::<Result<_, _>>()?,
                },
                quotient_value: Fp::new(qr.quotient_value),
                quotient_proof: crate::merkle::MerkleProof {
                    siblings: qr
                        .quotient_proof_siblings
                        .iter()
                        .map(|s| to_hash(s))
                        .collect::<Result<_, _>>()?,
                },
            })
        })
        .collect::<Result<_, String>>()?;

    let fri_query_proofs: Vec<crate::fri::FriQueryProof> = sp
        .fri_query_proofs
        .iter()
        .map(|qp| {
            Ok(crate::fri::FriQueryProof {
                layers: qp
                    .layers
                    .iter()
                    .map(|l| {
                        Ok(crate::fri::FriQueryLayer {
                            value: Fp::new(l.value),
                            sibling_value: Fp::new(l.sibling_value),
                            proof: crate::merkle::MerkleProof {
                                siblings: l
                                    .proof_siblings
                                    .iter()
                                    .map(|s| to_hash(s))
                                    .collect::<Result<_, _>>()?,
                            },
                            sibling_proof: crate::merkle::MerkleProof {
                                siblings: l
                                    .sibling_proof_siblings
                                    .iter()
                                    .map(|s| to_hash(s))
                                    .collect::<Result<_, _>>()?,
                            },
                        })
                    })
                    .collect::<Result<_, String>>()?,
            })
        })
        .collect::<Result<_, String>>()?;

    Ok(prover::StarkProof {
        trace_root,
        quotient_root,
        fri_proof: crate::fri::FriProof {
            layer_roots: sp
                .fri_layer_roots
                .iter()
                .map(|s| to_hash(s))
                .collect::<Result<_, _>>()?,
            final_layer: sp.fri_final_layer.iter().map(|v| Fp::new(*v)).collect(),
            query_proofs: fri_query_proofs,
        },
        query_responses,
        query_indices: sp.query_indices,
        program,
        outputs,
        trace_length: sp.trace_length,
        real_trace_length: sp.real_trace_length,
    })
}
