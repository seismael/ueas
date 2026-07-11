/* tslint:disable */
/* eslint-disable */

export function execute_ueas(s: string): string;

export function main(): void;

export function parse_ueas(s: string): string;

export function profile_complexity(s: string): string;

export function profile_hardware(s: string): string;

export function profile_memory(s: string): string;

export function transpile_ueas(s: string, t: string): string;

export function verify_crypto(s: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly execute_ueas: (a: number, b: number) => [number, number, number, number];
    readonly parse_ueas: (a: number, b: number) => [number, number, number, number];
    readonly profile_hardware: (a: number, b: number) => [number, number, number, number];
    readonly transpile_ueas: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly verify_crypto: (a: number, b: number) => [number, number, number, number];
    readonly main: () => void;
    readonly profile_memory: (a: number, b: number) => [number, number, number, number];
    readonly profile_complexity: (a: number, b: number) => [number, number, number, number];
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
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
