/* tslint:disable */
/* eslint-disable */

/**
 * Parse, lower, typecheck, MVS-check, and CT-check a UEAS v2.1 source string.
 */
export function ueas_check(source: string): string;

/**
 * Format UEAS v2.0 source code using the lossless CST formatter.
 */
export function ueas_format(source: string): string;

/**
 * Emit canonical HIR JSON for a UEAS v2.0 source string.
 */
export function ueas_hir_json(source: string): string;

/**
 * Execute `@Test` blocks or a specific zero-arg `entry` in `ueas-vm` and return real execution metrics.
 */
export function ueas_run(source: string, entry: string, step_budget: bigint): string;

/**
 * Transpile UEAS v2.0 source to a Track A, B, or C target (`rust`, `python`, `typescript`, `cpp`, `go`, `dafny`, `lean4`, `verus`, `quint`, `tla`, `latex`, `typst`).
 */
export function ueas_transpile(source: string, target: string): string;

/**
 * Run progressive verification (`"L0"`, `"L1"`, `"L2"`, `"L3"`) and return a SHA-256 certificate JSON.
 */
export function ueas_verify(source: string, level: string, fuzz_iterations: number): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly ueas_check: (a: number, b: number) => [number, number];
    readonly ueas_format: (a: number, b: number) => [number, number];
    readonly ueas_hir_json: (a: number, b: number) => [number, number];
    readonly ueas_run: (a: number, b: number, c: number, d: number, e: bigint) => [number, number];
    readonly ueas_transpile: (a: number, b: number, c: number, d: number) => [number, number];
    readonly ueas_verify: (a: number, b: number, c: number, d: number, e: number) => [number, number];
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
