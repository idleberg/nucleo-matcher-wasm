import { defineConfig } from 'vitest/config'
import wasm from 'vite-plugin-wasm'

export default defineConfig({
  plugins: [wasm()],
  test: {
    // forks avoids SharedArrayBuffer/Atomics conflicts between Worker threads and WASM
    pool: 'forks',
  },
})
