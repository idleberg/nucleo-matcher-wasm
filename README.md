# nucleo-matcher-wasm

Fast fuzzy finder, powered by [`nucleo-matcher`](https://crates.io/crates/nucleo-matcher) compiled to WebAssembly.

> `nucleo` is a highly performant fuzzy matcher written in rust. It aims to fill the same use case as `fzf` and `skim`. Compared to `fzf`, `nucleo` has a significantly faster matching algorithm (see [benchmarks](https://crates.io/crates/nucleo-matcher#benchmarks)).

## Installation

```sh
npm i nucleo-matcher-wasm
```

## Usage

```typescript
import { NucleoMatcher } from 'nucleo-matcher-wasm';

const nucleo = new NucleoMatcher([
    'some/path',
    'some/other/path',
    'even/one/more'
]);

nucleo.match('some');
```

## Benchmarks

This repository contains a benchmark script, testing `nucleo-matcher-wasm` against some popular fuzzy finder libraries for NodeJS. Run `pnpm bench` to see the results for your computer.

## Development

### Prerequisites

#### Rust & Cargo

Install via [rustup](https://rustup.rs/):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### wasm-pack

[wasm-pack](https://rustwasm.github.io/wasm-pack/) compiles the Rust crate to WebAssembly and generates JS/TS bindings.

```sh
cargo install wasm-pack
```

#### Node.js

Node.js ≥ 18 is required for running the playground scripts and publishing to npm.

### Building

Build the WASM package targeting Node.js:

```sh
wasm-pack build --target nodejs --out-dir dist
```

## License

This work is licensed under [Mozilla Public License 2.0
](LICENSE).
