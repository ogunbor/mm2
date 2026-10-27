/* tslint:disable */
/* eslint-disable */

/**
 * Verify AIR constraints on a trace.
 */
export function check_constraints(program_json: string): string;

/**
 * Generate a demo program (Nicomachus's theorem).
 */
export function demo_program(n: number, power: number): string;

/**
 * Execute a program and return the execution trace as JSON.
 */
export function execute_program(program_json: string): string;

/**
 * Execute a program without checking assertions.
 */
export function execute_program_unchecked(program_json: string): string;

/**
 * Generate a STARK proof for a program.
 */
export function generate_proof(program_json: string): string;

/**
 * Get ISA info (opcodes, register count, etc).
 */
export function get_isa_info(): string;

/**
 * Verify a STARK proof.
 */
export function verify_proof(proof_data: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly check_constraints: (a: number, b: number) => [number, number];
    readonly demo_program: (a: number, b: number) => [number, number];
    readonly execute_program: (a: number, b: number) => [number, number];
    readonly execute_program_unchecked: (a: number, b: number) => [number, number];
    readonly generate_proof: (a: number, b: number) => [number, number];
    readonly get_isa_info: () => [number, number];
    readonly verify_proof: (a: number, b: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
