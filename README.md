# nucleo-matcher-wasm

[![License](https://img.shields.io/github/license/idleberg/nucleo-matcher-wasm?style=for-the-badge)](LICENSE)
[![Version](https://img.shields.io/npm/v/nucleo-matcher-wasm?style=for-the-badge)](https://www.npmjs.org/package/nucleo-matcher-wasm)
[![Benchmarks](https://img.shields.io/github/actions/workflow/status/idleberg/nucleo-matcher-wasm/benchmark.yml?logo=nodedotjs&logoColor=white&style=for-the-badge)](https://github.com/idleberg/nucleo-matcher-wasm/actions/workflows/benchmark.yml)

Fast fuzzy finder, powered by [`nucleo-matcher`](https://crates.io/crates/nucleo-matcher) compiled to WebAssembly.

> `nucleo` is a highly performant fuzzy matcher written in rust. It aims to fill the same use case as `fzf` and `skim`. Compared to `fzf`, `nucleo` has a significantly faster matching algorithm (see [benchmarks](https://crates.io/crates/nucleo-matcher#benchmarks)).

## Installation

```sh
npm i nucleo-matcher-wasm
```

## Usage

```typescript
import { NucleoMatcher } from 'nucleo-matcher-wasm';

const items = [
    'src/components/Header.tsx',
    'src/utils/helpers.ts',
    'test/fixtures/data.json',
];

// Basic usage (defaults: case-insensitive, smart normalization)
const nucleo = new NucleoMatcher(items);

// With options (file path matching, smart case)
const nucleo = new NucleoMatcher(items, {
    matchPaths: true,
    caseMatching: 'smart',
});
```

### API

#### matchPattern

Usage: `matchPattern(pattern, options?)`

Match with fzf-like syntax (`^` prefix, `$` postfix, `'` substring, `!` negation):

```typescript
nucleo.matchPattern('header');
// → [['src/components/Header.tsx', 168]]

nucleo.matchPattern('^src comp');
// → [['src/components/Header.tsx', 168]]
```

#### matchLiteral

Usage:`matchLiteral(pattern, kind?, options?)`

Match literally (no special syntax parsing). `kind`: `"fuzzy"` (default), `"substring"`, `"prefix"`, `"postfix"`, `"exact"`:

```typescript
nucleo.matchLiteral('^src', 'fuzzy');
// Treats ^ as a literal character

nucleo.matchLiteral('test/', 'prefix');
// → [['test/fixtures/data.json', ...]]
```

#### matchPatternIndices

Usage: `matchPatternIndices(pattern, options?)` / `matchLiteralIndices(pattern, kind?, options?)`

Same as above but also returns matched character indices (for highlighting):

```typescript
nucleo.matchPatternIndices('header');
// → [['src/components/Header.tsx', 168, [15, 16, 17, 18, 19, 20]]]
```

#### setItems

Usage: `setItems(items)`

Replace the stored item list.

#### score

Usage: `score(pattern, haystack, options?)`

Score a single string without pre-loading items:

```typescript
nucleo.score('hlp', 'helpers.ts');
// → 96

nucleo.score('xyz', 'helpers.ts');
// → undefined
```

### Options

Options can be set in the constructor and the provided matching methods.

#### matchPaths

> [!WARNING]
>
> This option can only be set in the constructor.

Values: `boolean`  
Default: `false`  

Treat `/` and `\\` as word boundaries.

#### preferPrefix

> [!WARNING]
>
> This option can only be set in the constructor.

Values: `boolean`  
Default: `false`  

Boost matches near the start of the haystack.

#### caseMatching

Values: `"ignore" | "smart" | "respect"`  
Default: `"ignore"`  

Case sensitivity mode.

#### normalization

Values: `"smart" | "never"`  
Default: `"smart"`  

Unicode normalization mode

## Benchmarks

This repository contains a benchmark script, testing `nucleo-matcher-wasm` against some popular fuzzy finder libraries for NodeJS. Run `npm run bench` to see the results for your computer.

## Development

### Prerequisites

#### Rust & Cargo

Install via [rustup](https://rustup.rs/):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### wasm-pack

[wasm-pack](https://wasm-bindgen.github.io/wasm-pack/) compiles the Rust crate to WebAssembly and generates JS/TS bindings.

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
