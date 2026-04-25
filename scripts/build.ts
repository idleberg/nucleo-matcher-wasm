#!/usr/bin/env zx
import { $ } from 'zx'

const TEMP_DIR = '.build';
const OUT_DIR = 'dist';

await $`rm -rf ${OUT_DIR} ${TEMP_DIR}`

await $`wasm-pack build --no-pack --out-dir ${TEMP_DIR}/esm --release`
await $`wasm-pack build --no-pack --out-dir ${TEMP_DIR}/cjs --target nodejs --release`

await $`mkdir ./${OUT_DIR}`

await $`cp ${TEMP_DIR}/cjs/nucleo_wasm.js ${OUT_DIR}/nucleo_wasm.cjs`

await $`cp ${TEMP_DIR}/esm/nucleo_wasm.js ${OUT_DIR}/nucleo_wasm.mjs`

await $`cp ${TEMP_DIR}/esm/nucleo_wasm.d.ts ${OUT_DIR}/nucleo_wasm.d.ts`
await $`cp ${TEMP_DIR}/esm/nucleo_wasm_bg.js ${OUT_DIR}/nucleo_wasm_bg.js`
await $`cp ${TEMP_DIR}/esm/nucleo_wasm_bg.wasm ${OUT_DIR}/nucleo_wasm_bg.wasm`
await $`cp ${TEMP_DIR}/esm/nucleo_wasm_bg.wasm.d.ts ${OUT_DIR}/nucleo_wasm_bg.wasm.d.ts`
