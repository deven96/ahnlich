/* tslint:disable */
/* eslint-disable */

export class AhnlichDB {
    free(): void;
    [Symbol.dispose](): void;
    create_non_linear_algorithm_index(request_bytes: Uint8Array): Uint8Array;
    create_pred_index(request_bytes: Uint8Array): Uint8Array;
    create_store(request_bytes: Uint8Array): Uint8Array;
    del_key(request_bytes: Uint8Array): Uint8Array;
    del_pred(request_bytes: Uint8Array): Uint8Array;
    drop_non_linear_algorithm_index(request_bytes: Uint8Array): Uint8Array;
    drop_pred_index(request_bytes: Uint8Array): Uint8Array;
    drop_schema(request_bytes: Uint8Array): Uint8Array;
    drop_store(request_bytes: Uint8Array): Uint8Array;
    export_snapshot(): Uint8Array;
    get_key(request_bytes: Uint8Array): Uint8Array;
    get_pred(request_bytes: Uint8Array): Uint8Array;
    get_sim_n(request_bytes: Uint8Array): Uint8Array;
    get_store(request_bytes: Uint8Array): Uint8Array;
    import_snapshot(snapshot_bytes: Uint8Array): void;
    list_stores(request_bytes: Uint8Array): Uint8Array;
    constructor();
    set(request_bytes: Uint8Array): Uint8Array;
    upsert(request_bytes: Uint8Array): Uint8Array;
}

export function init(): void;

export function initThreadPool(num_threads: number): Promise<any>;

export class wbg_rayon_PoolBuilder {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    build(): void;
    numThreads(): number;
    receiver(): number;
}

export function wbg_rayon_start_worker(receiver: number): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly __wbg_ahnlichdb_free: (a: number, b: number) => void;
    readonly ahnlichdb_create_non_linear_algorithm_index: (a: number, b: number, c: number) => [number, number, number, number];
    readonly ahnlichdb_create_pred_index: (a: number, b: number, c: number) => [number, number, number, number];
    readonly ahnlichdb_create_store: (a: number, b: number, c: number) => [number, number, number, number];
    readonly ahnlichdb_del_key: (a: number, b: number, c: number) => [number, number, number, number];
    readonly ahnlichdb_del_pred: (a: number, b: number, c: number) => [number, number, number, number];
    readonly ahnlichdb_drop_non_linear_algorithm_index: (a: number, b: number, c: number) => [number, number, number, number];
    readonly ahnlichdb_drop_pred_index: (a: number, b: number, c: number) => [number, number, number, number];
    readonly ahnlichdb_drop_schema: (a: number, b: number, c: number) => [number, number, number, number];
    readonly ahnlichdb_drop_store: (a: number, b: number, c: number) => [number, number, number, number];
    readonly ahnlichdb_export_snapshot: (a: number) => [number, number, number, number];
    readonly ahnlichdb_get_key: (a: number, b: number, c: number) => [number, number, number, number];
    readonly ahnlichdb_get_pred: (a: number, b: number, c: number) => [number, number, number, number];
    readonly ahnlichdb_get_sim_n: (a: number, b: number, c: number) => [number, number, number, number];
    readonly ahnlichdb_get_store: (a: number, b: number, c: number) => [number, number, number, number];
    readonly ahnlichdb_import_snapshot: (a: number, b: number, c: number) => [number, number];
    readonly ahnlichdb_list_stores: (a: number, b: number, c: number) => [number, number, number, number];
    readonly ahnlichdb_new: () => number;
    readonly ahnlichdb_set: (a: number, b: number, c: number) => [number, number, number, number];
    readonly ahnlichdb_upsert: (a: number, b: number, c: number) => [number, number, number, number];
    readonly init: () => void;
    readonly __wbg_wbg_rayon_poolbuilder_free: (a: number, b: number) => void;
    readonly initThreadPool: (a: number) => any;
    readonly wbg_rayon_poolbuilder_build: (a: number) => void;
    readonly wbg_rayon_poolbuilder_numThreads: (a: number) => number;
    readonly wbg_rayon_poolbuilder_receiver: (a: number) => number;
    readonly wbg_rayon_start_worker: (a: number) => void;
    readonly memory: WebAssembly.Memory;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_thread_destroy: (a?: number, b?: number, c?: number) => void;
    readonly __wbindgen_start: (a: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput, memory?: WebAssembly.Memory, thread_stack_size?: number }} module - Passing `SyncInitInput` directly is deprecated.
 * @param {WebAssembly.Memory} memory - Deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput, memory?: WebAssembly.Memory, thread_stack_size?: number } | SyncInitInput, memory?: WebAssembly.Memory): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput>, memory?: WebAssembly.Memory, thread_stack_size?: number }} module_or_path - Passing `InitInput` directly is deprecated.
 * @param {WebAssembly.Memory} memory - Deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput>, memory?: WebAssembly.Memory, thread_stack_size?: number } | InitInput | Promise<InitInput>, memory?: WebAssembly.Memory): Promise<InitOutput>;
