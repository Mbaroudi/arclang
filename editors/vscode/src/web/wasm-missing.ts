// Stands in for wasm/pkg/arclang_wasm.js when the WebAssembly compiler was not built (see esbuild.mjs).
const missing = (): never => {
  throw new Error('The ArcLang compiler for the browser is not built: run `wasm-pack build` in wasm/ (see docs/WEB_PLAYGROUND.md), then build the extension again.');
};

export const initSync = missing;
export const compile = missing;
export const format = missing;
export const edit = missing;
export const metamodel = missing;
export const rules = missing;
export const version = missing;
export const transition = missing;
export const import_sysml = missing;
export const import_sysml_files = missing;
export const workbench = missing;
