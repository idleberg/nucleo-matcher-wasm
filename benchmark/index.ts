import { NucleoMatcher } from "../dist/nucleo_wasm.js";
import fuzzaldrin from "fuzzaldrin-plus";
import fuzzaldrinOriginal from "fuzzaldrin";
import Fuse from "fuse.js";
import * as fzy from "fzy.js";
import fzi from "fzi";
import { Fzf } from "fzf";
import fuzzy from "fuzzy";
import FuzzySearch from "fuzzy-search";
import { Searcher as FastFuzzySearcher } from "fast-fuzzy";

// Generate a realistic dataset of file paths
function generatePaths(count: number): string[] {
  const dirs = [
    "src/components", "src/utils", "src/hooks", "src/pages", "src/api",
    "src/services", "src/models", "src/store", "src/config", "src/types",
    "src/middleware", "src/validators", "src/controllers", "src/views",
    "lib/core", "lib/plugins", "lib/helpers", "lib/adapters", "lib/auth",
    "test/unit", "test/integration", "test/e2e", "test/fixtures",
    "packages/ui/src", "packages/shared/src", "packages/cli/src",
  ];
  const names = [
    "index", "utils", "helpers", "types", "config", "constants",
    "Header", "Footer", "Sidebar", "Navigation", "SearchBar", "Modal",
    "Button", "Input", "Select", "Dropdown", "Tooltip", "Avatar",
    "useAuth", "useTheme", "useModal", "useFetch", "useDebounce",
    "Dashboard", "Settings", "Profile", "Analytics", "UserList",
    "apiClient", "httpService", "authMiddleware", "validator",
    "formatDate", "parseQuery", "buildUrl", "mergeConfig",
  ];
  const exts = [".ts", ".tsx", ".js", ".jsx", ".test.ts", ".spec.ts"];
  const paths: string[] = [];
  for (let i = 0; i < count; i++) {
    const dir = dirs[i % dirs.length];
    const name = names[i % names.length];
    const suffix = i > names.length ? String(Math.floor(i / names.length)) : "";
    const ext = exts[i % exts.length];
    paths.push(`${dir}/${name}${suffix}${ext}`);
  }
  return paths;
}

function bench(label: string, fn: () => void, iterations: number): { label: string; perIter: number; elapsed: number; iterations: number } {
  // warmup
  for (let i = 0; i < 5; i++) fn();

  const start = performance.now();
  for (let i = 0; i < iterations; i++) fn();
  const elapsed = performance.now() - start;
  const perIter = elapsed / iterations;
  return { label, perIter, elapsed, iterations };
}

function printResults(results: ReturnType<typeof bench>[]): void {
  results.sort((a, b) => a.perIter - b.perIter);
  for (const r of results) {
    console.log(`  ${r.label.padEnd(20)} ${r.perIter.toFixed(3).padStart(8)} ms/iter  (${r.iterations} iters, ${r.elapsed.toFixed(0)} ms total)`);
  }
}

const patterns = ["side", "src/comp", "useFe", "modal", "test/fix"];
console.clear();

for (const size of [100, 1_000, 10_000]) {
  const items = generatePaths(size);
  const iterations = size <= 1000 ? 500 : 50;

  console.log(`\n${"=".repeat(70)}`);
  console.log(`Dataset: ${size.toLocaleString()} items, ${iterations} iterations per matcher`);
  console.log(`${"=".repeat(70)}`);

  const totals = new Map<string, number>();

  for (const pattern of patterns) {
    console.log(`\nPattern: '${pattern}'`);
    const results = [];

    // nucleo
    const nucleoStored = new NucleoMatcher(items);
    results.push(bench("nucleo (pattern)", () => {
      nucleoStored.matchPattern(pattern);
    }, iterations));

    // nucleo matchLiteral
    results.push(bench("nucleo (literal)", () => {
      nucleoStored.matchLiteral(pattern);
    }, iterations));

    // fuzzaldrin
    results.push(bench("fuzzaldrin", () => {
      fuzzaldrinOriginal.filter(items, pattern);
    }, iterations));

    // fuzzaldrin-plus
    results.push(bench("fuzzaldrin-plus", () => {
      fuzzaldrin.filter(items, pattern);
    }, iterations));

    // fzy.js
    results.push(bench("fzy.js", () => {
      items
        .filter((item) => fzy.hasMatch(pattern, item))
        .map((item) => ({ item, score: fzy.score(pattern, item) }))
        .sort((a, b) => b.score - a.score);
    }, iterations));

    // fzi
    results.push(bench("fzi", () => {
      fzi.search(pattern, items);
    }, iterations));

    // fzf
    const fzf = new Fzf(items);
    results.push(bench("fzf", () => {
      fzf.find(pattern);
    }, iterations));

    // fuse.js
    const fuse = new Fuse(items, { includeScore: true, threshold: 0.6 });
    results.push(bench("fuse.js", () => {
      fuse.search(pattern);
    }, iterations));

    // fuzzy
    results.push(bench("fuzzy", () => {
      fuzzy.filter(pattern, items);
    }, iterations));

    // fuzzy-search
    const fuzzySearch = new FuzzySearch(items);
    results.push(bench("fuzzy-search", () => {
      fuzzySearch.search(pattern);
    }, iterations));

    // fast-fuzzy
    const fastFuzzy = new FastFuzzySearcher(items);
    results.push(bench("fast-fuzzy", () => {
      fastFuzzy.search(pattern);
    }, iterations));

    printResults(results);
    for (const r of results) {
      totals.set(r.label, (totals.get(r.label) ?? 0) + r.perIter);
    }
    nucleoStored.free();
  }

  console.log(`\nAverages across all patterns:`);
  const avgs = [...totals.entries()]
    .map(([label, total]) => ({ label, avg: total / patterns.length }))
    .sort((a, b) => a.avg - b.avg);
  for (const { label, avg } of avgs) {
    console.log(`  ${label.padEnd(20)} ${avg.toFixed(3).padStart(8)} ms/iter`);
  }
}
