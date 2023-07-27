// This file is for loading the WASM module produced by `wasm-pack` for the
// "web" target. The reason why we don't use the default "bundler" target is
// that the WASM support in webpack v4 always mangles the module and produces
// the file in different filenames. When the module is used on both the web and
// webworker targets, this causes the module to be needlessly duplicated,
// leading to the client having to download the module twice. By bypassing
// webpack's WASM support and loading it the "manual" way, we stop webpack from
// mangling the WASM module so that the module won't be duplicated.

import wasmInit, * as wasm from "./web_wasm.js";

const wasmLoadPromise = wasmInit();
const wasmImport = wasmLoadPromise.then(_wasmInternal => wasm);

export { wasm, wasmImport, wasmLoadPromise };
