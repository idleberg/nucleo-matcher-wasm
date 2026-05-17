# nucleo-matcher-wasm

[![License](https://img.shields.io/github/license/idleberg/nucleo-matcher-wasm?style=for-the-badge)](LICENSE)
[![Version](https://img.shields.io/npm/v/nucleo-matcher-wasm?style=for-the-badge)](https://www.npmjs.org/package/nucleo-matcher-wasm)
[![CI](https://img.shields.io/github/actions/workflow/status/idleberg/nucleo-matcher-wasm/test.yml?logo=nodedotjs&logoColor=white&style=for-the-badge)](https://github.com/idleberg/nucleo-matcher-wasm/actions/workflows/test.yml)

Fast fuzzy finder, powered by [`nucleo-matcher`](https://crates.io/crates/nucleo-matcher) compiled to WebAssembly.

> `nucleo` is a highly performant fuzzy matcher written in rust. It aims to fill the same use case as `fzf` and `skim`. Compared to `fzf`, `nucleo` has a significantly faster matching algorithm. It is used in the [helix-editor](https://github.com/helix-editor/helix) and therefore has a large user base with lots of real world testing. The core matcher implementation is considered complete and is unlikely to see major changes

## Installation

```sh
npm i nucleo-matcher-wasm
```

## Usage

```typescript
import { NucleoMatcher } from 'nucleo-matcher-wasm';

const items = [
    'src/components/Header.svelte',
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
// → [['src/components/Header.svelte', 168]]

nucleo.matchPattern('^src comp');
// → [['src/components/Header.svelte', 168]]
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
// → [['src/components/Header.svelte', 168, [15, 16, 17, 18, 19, 20]]]
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

> [!NOTE]
>
> This option can only be set in the constructor (not per-call).

Values: `boolean`  
Default: `false`  

Treat `/` and `\\` as word boundaries.

#### preferPrefix

> [!NOTE]
>
> This option can only be set in the constructor (not per-call).

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

#### maxResults

> [!NOTE]
>
> This option can only be set per-call (not in the constructor).

Values: `number`  
Default: `undefined`  

Cap the result set to the top N matches by score. Skips marshaling discarded results across the WebAssembly boundary, and uses a bounded heap internally rather than scoring + full-sorting all candidates.

```typescript
nucleo.matchPattern('hdr', { maxResults: 50 });
```

## Benchmarks

This repository contains a benchmark script, testing `nucleo-matcher-wasm` against some popular fuzzy finder libraries for NodeJS. Here's the gist of it on an Apple M2 Pro with 32GB of RAM:

```
======================================================================
Dataset: 100 items, 500 iterations per matcher
======================================================================

Averages across all patterns:
  nucleo (literal)        0.005 ms/iter
  nucleo (pattern)        0.007 ms/iter
  fuzzy-search            0.009 ms/iter
  fzy.js                  0.016 ms/iter
  fuzzaldrin-plus         0.017 ms/iter
  fzi                     0.024 ms/iter
  fuzzy                   0.030 ms/iter
  fuzzaldrin              0.031 ms/iter
  fzf                     0.045 ms/iter
  fuse.js                 0.174 ms/iter
  fast-fuzzy              0.514 ms/iter

======================================================================
Dataset: 1,000 items, 500 iterations per matcher
======================================================================

Averages across all patterns:
  nucleo (literal)        0.046 ms/iter
  nucleo (pattern)        0.047 ms/iter
  fuzzy-search            0.087 ms/iter
  fzy.js                  0.175 ms/iter
  fuzzaldrin-plus         0.187 ms/iter
  fzi                     0.265 ms/iter
  fuzzaldrin              0.296 ms/iter
  fuzzy                   0.307 ms/iter
  fzf                     0.430 ms/iter
  fast-fuzzy              0.535 ms/iter
  fuse.js                 1.743 ms/iter

======================================================================
Dataset: 10,000 items, 50 iterations per matcher
======================================================================

Averages across all patterns:
  nucleo (literal)        0.467 ms/iter
  nucleo (pattern)        0.489 ms/iter
  fuzzy-search            0.886 ms/iter
  fzy.js                  1.852 ms/iter
  fuzzaldrin-plus         1.972 ms/iter
  fzi                     2.770 ms/iter
  fuzzaldrin              2.994 ms/iter
  fuzzy                   3.210 ms/iter
  fast-fuzzy              4.120 ms/iter
  fzf                     4.262 ms/iter
  fuse.js                17.832 ms/iter
```

Try `npm run bench` to see how well the benchmark performs on your target environment.

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
mise run build
```

## License

This work is licensed under [Mozilla Public License 2.0
](LICENSE).
