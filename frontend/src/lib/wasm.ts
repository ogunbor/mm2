// WASM bridge — lazy-loads the WASM module via fetch from /public
let wasmModule: any = null;
let initialized = false;
let initPromise: Promise<any> | null = null;

export async function getWasm() {
  if (initialized) return wasmModule;
  if (initPromise) return initPromise;

  initPromise = (async () => {
    // Dynamic import of the JS glue code
    const mod = await import("../../pkg/mm2.js");
    // Init with explicit URL to the .wasm file in /public
    await mod.default("/mm2_bg.wasm");
    wasmModule = mod;
    initialized = true;
    return mod;
  })();

  return initPromise;
}

export interface JsInstruction {
  opcode: string;
  rd: number;
  rs1: number;
  rs2: number;
  imm: number;
}

export interface JsTraceRow {
  clk: number;
  pc: number;
  opcode: number;
  opcode_name: string;
  rd: number;
  rs1: number;
  rs2: number;
  imm: number;
  regs: number[];
  rs1_val: number;
  rs2_val: number;
  write_val: number;
  is_padding: boolean;
}

export interface ExecutionResult {
  rows: JsTraceRow[];
  real_len: number;
  padded_len: number;
  success: boolean;
  error: string | null;
}

export interface ConstraintResult {
  valid: boolean;
  error_row: number | null;
  error_constraint: number | null;
}

export interface ProofResult {
  success: boolean;
  error: string | null;
  trace_root: string | null;
  quotient_root: string | null;
  num_queries: number | null;
  trace_length: number | null;
  fri_layers: number | null;
  outputs: number[] | null;
  proof_data: string | null;
}

export interface VerifyStep {
  name: string;
  passed: boolean;
  detail: string;
}

export interface VerifyResult {
  valid: boolean;
  error: string | null;
  steps: VerifyStep[];
}

export interface OpcodeInfo {
  code: number;
  name: string;
  format: string;
  description: string;
}

export interface IsaInfo {
  field: string;
  prime: string;
  num_registers: number;
  opcodes: OpcodeInfo[];
  num_trace_columns: number;
  num_constraints: number;
  constraint_degree: number;
  blowup_factor: number;
  num_fri_queries: number;
}

// High-level API functions

export async function executeProgram(program: JsInstruction[]): Promise<ExecutionResult> {
  const wasm = await getWasm();
  const result = wasm.execute_program(JSON.stringify(program));
  return JSON.parse(result);
}

export async function executeProgramUnchecked(program: JsInstruction[]): Promise<ExecutionResult> {
  const wasm = await getWasm();
  const result = wasm.execute_program_unchecked(JSON.stringify(program));
  return JSON.parse(result);
}

export async function checkConstraints(program: JsInstruction[]): Promise<ConstraintResult> {
  const wasm = await getWasm();
  const result = wasm.check_constraints(JSON.stringify(program));
  return JSON.parse(result);
}

export async function generateProof(program: JsInstruction[]): Promise<ProofResult> {
  const wasm = await getWasm();
  const result = wasm.generate_proof(JSON.stringify(program));
  return JSON.parse(result);
}

export async function verifyProof(proofData: string): Promise<VerifyResult> {
  const wasm = await getWasm();
  const result = wasm.verify_proof(proofData);
  return JSON.parse(result);
}

export async function getDemoProgram(n: number, power: number): Promise<JsInstruction[]> {
  const wasm = await getWasm();
  const result = wasm.demo_program(n, power);
  return JSON.parse(result);
}

export async function getIsaInfo(): Promise<IsaInfo> {
  const wasm = await getWasm();
  const result = wasm.get_isa_info();
  return JSON.parse(result);
}

// Helper: format instruction as assembly string
export function formatInstruction(inst: JsInstruction): string {
  switch (inst.opcode) {
    case "NOP": return "NOP";
    case "HALT": return "HALT";
    case "IMM": return `IMM r${inst.rd}, ${inst.imm}`;
    case "ADD": return `ADD r${inst.rd}, r${inst.rs1}, r${inst.rs2}`;
    case "MUL": return `MUL r${inst.rd}, r${inst.rs1}, r${inst.rs2}`;
    case "SUB": return `SUB r${inst.rd}, r${inst.rs1}, r${inst.rs2}`;
    case "ADDI": return `ADDI r${inst.rd}, r${inst.rs1}, ${inst.imm}`;
    case "ASSERT_EQ": return `ASSERT_EQ r${inst.rs1}, r${inst.rs2}`;
    default: return `${inst.opcode} ???`;
  }
}

// Helper: parse assembly text into instructions
export function parseAssembly(text: string): JsInstruction[] {
  const lines = text.split("\n").map(l => l.trim()).filter(l => l && !l.startsWith("//") && !l.startsWith("#"));
  const instructions: JsInstruction[] = [];

  for (const line of lines) {
    const parts = line.replace(/,/g, " ").split(/\s+/);
    const op = parts[0].toUpperCase();
    const parseReg = (s: string): number => {
      const m = s.match(/^r(\d+)$/i);
      if (!m) throw new Error(`Invalid register: ${s}`);
      return parseInt(m[1]);
    };
    const parseImm = (s: string): number => parseInt(s);

    switch (op) {
      case "NOP":
        instructions.push({ opcode: "NOP", rd: 0, rs1: 0, rs2: 0, imm: 0 });
        break;
      case "HALT":
        instructions.push({ opcode: "HALT", rd: 0, rs1: 0, rs2: 0, imm: 0 });
        break;
      case "IMM":
        instructions.push({ opcode: "IMM", rd: parseReg(parts[1]), rs1: 0, rs2: 0, imm: parseImm(parts[2]) });
        break;
      case "ADD":
        instructions.push({ opcode: "ADD", rd: parseReg(parts[1]), rs1: parseReg(parts[2]), rs2: parseReg(parts[3]), imm: 0 });
        break;
      case "MUL":
        instructions.push({ opcode: "MUL", rd: parseReg(parts[1]), rs1: parseReg(parts[2]), rs2: parseReg(parts[3]), imm: 0 });
        break;
      case "SUB":
        instructions.push({ opcode: "SUB", rd: parseReg(parts[1]), rs1: parseReg(parts[2]), rs2: parseReg(parts[3]), imm: 0 });
        break;
      case "ADDI":
        instructions.push({ opcode: "ADDI", rd: parseReg(parts[1]), rs1: parseReg(parts[2]), rs2: 0, imm: parseImm(parts[3]) });
        break;
      case "ASSERT_EQ":
        instructions.push({ opcode: "ASSERT_EQ", rd: 0, rs1: parseReg(parts[1]), rs2: parseReg(parts[2]), imm: 0 });
        break;
      default:
        throw new Error(`Unknown instruction: ${op}`);
    }
  }

  return instructions;
}
