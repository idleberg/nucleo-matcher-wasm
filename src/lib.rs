use std::cmp::Reverse;
use std::collections::BinaryHeap;

use nucleo_matcher::pattern::Pattern;
use nucleo_matcher::{Config, Matcher, Utf32Str, Utf32String};
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Tsify, Serialize, Deserialize, Default, Clone, Copy)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "lowercase")]
pub enum CaseMatching {
    #[default]
    Ignore,
    Smart,
    Respect,
}

#[derive(Tsify, Serialize, Deserialize, Default, Clone, Copy)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "lowercase")]
pub enum Normalization {
    #[default]
    Smart,
    Never,
}

#[derive(Tsify, Serialize, Deserialize, Default, Clone, Copy)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "lowercase")]
pub enum AtomKind {
    #[default]
    Fuzzy,
    Substring,
    Prefix,
    Postfix,
    Exact,
}

impl From<CaseMatching> for nucleo_matcher::pattern::CaseMatching {
    fn from(val: CaseMatching) -> Self {
        match val {
            CaseMatching::Ignore => Self::Ignore,
            CaseMatching::Smart => Self::Smart,
            CaseMatching::Respect => Self::Respect,
        }
    }
}

impl From<Normalization> for nucleo_matcher::pattern::Normalization {
    fn from(val: Normalization) -> Self {
        match val {
            Normalization::Smart => Self::Smart,
            Normalization::Never => Self::Never,
        }
    }
}

impl From<AtomKind> for nucleo_matcher::pattern::AtomKind {
    fn from(val: AtomKind) -> Self {
        match val {
            AtomKind::Fuzzy => Self::Fuzzy,
            AtomKind::Substring => Self::Substring,
            AtomKind::Prefix => Self::Prefix,
            AtomKind::Postfix => Self::Postfix,
            AtomKind::Exact => Self::Exact,
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Default)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct MatcherOptions {
    /// Treat `/` and `\` as word boundaries (default: false)
    #[serde(default)]
    pub match_paths: bool,
    /// Boost matches near the start of the haystack (default: false)
    #[serde(default)]
    pub prefer_prefix: bool,
    /// Case sensitivity mode (default: "ignore")
    #[serde(default)]
    pub case_matching: CaseMatching,
    /// Unicode normalization mode (default: "smart")
    #[serde(default)]
    pub normalization: Normalization,
}

#[derive(Tsify, Serialize, Deserialize, Default)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct MatchOptions {
    /// Case sensitivity mode – overrides the constructor default for this call
    pub case_matching: Option<CaseMatching>,
    /// Unicode normalization mode – overrides the constructor default for this call
    pub normalization: Option<Normalization>,
    /// Cap the result set to the top N matches by score (skips marshaling the rest)
    pub max_results: Option<u32>,
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND: &str = r#"
export type MatchResult = [item: string, score: number];
export type MatchResultWithIndices = [item: string, score: number, indices: number[]];
export type IndexedMatchResult = { indices: Uint32Array; scores: Uint32Array };
"#;

fn scored_to_js(items: &[String], scored: &[(usize, u32)]) -> JsValue {
    let result = js_sys::Array::new();
    for &(idx, score) in scored {
        let pair = js_sys::Array::new();
        pair.push(&JsValue::from_str(&items[idx]));
        pair.push(&JsValue::from_f64(score as f64));
        result.push(&pair);
    }
    result.into()
}

fn scored_to_indexed_js(scored: &[(usize, u32)]) -> JsValue {
    let (indices_buf, scores_buf): (Vec<u32>, Vec<u32>) = scored
        .iter()
        .map(|&(idx, score)| (idx as u32, score))
        .unzip();
    let indices_arr = js_sys::Uint32Array::from(indices_buf.as_slice());
    let scores_arr = js_sys::Uint32Array::from(scores_buf.as_slice());
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &JsValue::from_str("indices"), &indices_arr).unwrap();
    js_sys::Reflect::set(&obj, &JsValue::from_str("scores"), &scores_arr).unwrap();
    obj.into()
}

struct PatternCache {
    text: String,
    case_matching: nucleo_matcher::pattern::CaseMatching,
    normalization: nucleo_matcher::pattern::Normalization,
    atom_kind: Option<nucleo_matcher::pattern::AtomKind>,
    pattern: Pattern,
}

#[wasm_bindgen]
pub struct NucleoMatcher {
    matcher: Matcher,
    items: Vec<String>,
    haystacks: Vec<Utf32String>,
    case_matching: nucleo_matcher::pattern::CaseMatching,
    normalization: nucleo_matcher::pattern::Normalization,
    cache: Option<PatternCache>,
}

#[wasm_bindgen]
impl NucleoMatcher {
    /// Create a new matcher with items and optional configuration.
    #[wasm_bindgen(constructor)]
    pub fn new(items: Vec<String>, options: Option<MatcherOptions>) -> NucleoMatcher {
        let opts = options.unwrap_or_default();

        let mut config = Config::DEFAULT;
        if opts.match_paths {
            config = config.match_paths();
        }
        if opts.prefer_prefix {
            config.prefer_prefix = true;
        }

        let haystacks = items.iter().map(|s| Utf32String::from(s.as_str())).collect();

        NucleoMatcher {
            matcher: Matcher::new(config),
            items,
            haystacks,
            case_matching: opts.case_matching.into(),
            normalization: opts.normalization.into(),
            cache: None,
        }
    }

    /// Replace the stored item list.
    #[wasm_bindgen(js_name = "setItems")]
    pub fn set_items(&mut self, items: Vec<String>) {
        self.haystacks = items.iter().map(|s| Utf32String::from(s.as_str())).collect();
        self.items = items;
    }

    /// Match a pattern (with fzf-like syntax: `^`, `$`, `'`, `!`) against stored items.
    /// Returns `[item, score]` pairs sorted by score.
    #[wasm_bindgen(js_name = "matchPattern")]
    pub fn match_pattern(&mut self, pattern: &str, options: Option<MatchOptions>) -> JsValue {
        let (cm, norm) = self.resolve_options(&options);
        let max = max_results(&options);
        self.ensure_pattern(pattern, cm, norm, None);
        let pat = &self.cache.as_ref().unwrap().pattern;
        let scored = score_all(pat, &self.haystacks, &mut self.matcher, max);
        scored_to_js(&self.items, &scored)
    }

    /// Match a literal pattern against stored items using the specified matching kind.
    /// Special characters are treated literally (no fzf syntax parsing).
    #[wasm_bindgen(js_name = "matchLiteral")]
    pub fn match_literal(&mut self, pattern: &str, kind: Option<AtomKind>, options: Option<MatchOptions>) -> JsValue {
        let (cm, norm) = self.resolve_options(&options);
        let max = max_results(&options);
        let atom_kind: nucleo_matcher::pattern::AtomKind = kind.unwrap_or_default().into();
        self.ensure_pattern(pattern, cm, norm, Some(atom_kind));
        let pat = &self.cache.as_ref().unwrap().pattern;
        let scored = score_all(pat, &self.haystacks, &mut self.matcher, max);
        scored_to_js(&self.items, &scored)
    }

    /// Match a pattern (with fzf-like syntax) and return parallel typed arrays of
    /// haystack indices + scores. Skips copying matched strings across the WASM
    /// boundary — the caller looks up `items[indices[i]]` on the JS side.
    #[wasm_bindgen(js_name = "matchPatternIndexed")]
    pub fn match_pattern_indexed(&mut self, pattern: &str, options: Option<MatchOptions>) -> JsValue {
        let (cm, norm) = self.resolve_options(&options);
        let max = max_results(&options);
        self.ensure_pattern(pattern, cm, norm, None);
        let pat = &self.cache.as_ref().unwrap().pattern;
        let scored = score_all(pat, &self.haystacks, &mut self.matcher, max);
        scored_to_indexed_js(&scored)
    }

    /// Match a literal pattern and return parallel typed arrays of haystack
    /// indices + scores. See `matchPatternIndexed` for the marshaling rationale.
    #[wasm_bindgen(js_name = "matchLiteralIndexed")]
    pub fn match_literal_indexed(&mut self, pattern: &str, kind: Option<AtomKind>, options: Option<MatchOptions>) -> JsValue {
        let (cm, norm) = self.resolve_options(&options);
        let max = max_results(&options);
        let atom_kind: nucleo_matcher::pattern::AtomKind = kind.unwrap_or_default().into();
        self.ensure_pattern(pattern, cm, norm, Some(atom_kind));
        let pat = &self.cache.as_ref().unwrap().pattern;
        let scored = score_all(pat, &self.haystacks, &mut self.matcher, max);
        scored_to_indexed_js(&scored)
    }

    /// Match a pattern (with fzf-like syntax) and return match indices for highlighting.
    /// Returns `[item, score, indices[]]` triples sorted by score.
    #[wasm_bindgen(js_name = "matchPatternIndices")]
    pub fn match_pattern_indices(&mut self, pattern: &str, options: Option<MatchOptions>) -> JsValue {
        let (cm, norm) = self.resolve_options(&options);
        let max = max_results(&options);
        self.ensure_pattern(pattern, cm, norm, None);
        let pat = &self.cache.as_ref().unwrap().pattern;
        match_with_indices(pat, &self.items, &self.haystacks, &mut self.matcher, max)
    }

    /// Match a literal pattern and return match indices for highlighting.
    #[wasm_bindgen(js_name = "matchLiteralIndices")]
    pub fn match_literal_indices(&mut self, pattern: &str, kind: Option<AtomKind>, options: Option<MatchOptions>) -> JsValue {
        let (cm, norm) = self.resolve_options(&options);
        let max = max_results(&options);
        let atom_kind: nucleo_matcher::pattern::AtomKind = kind.unwrap_or_default().into();
        self.ensure_pattern(pattern, cm, norm, Some(atom_kind));
        let pat = &self.cache.as_ref().unwrap().pattern;
        match_with_indices(pat, &self.items, &self.haystacks, &mut self.matcher, max)
    }

    /// Score a single haystack string against a pattern (with fzf-like syntax).
    /// Returns the score as a number, or `undefined` if no match.
    #[wasm_bindgen]
    pub fn score(&mut self, pattern: &str, haystack: &str, options: Option<MatchOptions>) -> JsValue {
        let (cm, norm) = self.resolve_options(&options);
        self.ensure_pattern(pattern, cm, norm, None);
        let pat = &self.cache.as_ref().unwrap().pattern;
        let mut buf = Vec::new();
        let haystack_utf32 = Utf32Str::new(haystack, &mut buf);
        match pat.score(haystack_utf32, &mut self.matcher) {
            Some(score) => JsValue::from_f64(score as f64),
            None => JsValue::UNDEFINED,
        }
    }
}

impl NucleoMatcher {
    fn resolve_options(
        &self,
        options: &Option<MatchOptions>,
    ) -> (
        nucleo_matcher::pattern::CaseMatching,
        nucleo_matcher::pattern::Normalization,
    ) {
        let cm = options
            .as_ref()
            .and_then(|o| o.case_matching)
            .map(Into::into)
            .unwrap_or(self.case_matching);
        let norm = options
            .as_ref()
            .and_then(|o| o.normalization)
            .map(Into::into)
            .unwrap_or(self.normalization);
        (cm, norm)
    }

    fn ensure_pattern(
        &mut self,
        text: &str,
        cm: nucleo_matcher::pattern::CaseMatching,
        norm: nucleo_matcher::pattern::Normalization,
        atom_kind: Option<nucleo_matcher::pattern::AtomKind>,
    ) {
        if let Some(c) = &self.cache {
            if c.atom_kind == atom_kind
                && c.case_matching == cm
                && c.normalization == norm
                && c.text == text
            {
                return;
            }
        }
        let pattern = match atom_kind {
            None => {
                if let Some(c) = self.cache.as_mut() {
                    if c.atom_kind.is_none() {
                        c.pattern.reparse(text, cm, norm);
                        c.text.clear();
                        c.text.push_str(text);
                        c.case_matching = cm;
                        c.normalization = norm;
                        return;
                    }
                }
                Pattern::parse(text, cm, norm)
            }
            Some(kind) => Pattern::new(text, cm, norm, kind),
        };
        self.cache = Some(PatternCache {
            text: text.to_string(),
            case_matching: cm,
            normalization: norm,
            atom_kind,
            pattern,
        });
    }
}

fn max_results(options: &Option<MatchOptions>) -> Option<usize> {
    options.as_ref().and_then(|o| o.max_results).map(|n| n as usize)
}

fn score_all(
    pat: &Pattern,
    haystacks: &[Utf32String],
    matcher: &mut Matcher,
    max: Option<usize>,
) -> Vec<(usize, u32)> {
    if pat.atoms.is_empty() {
        let n = max.map_or(haystacks.len(), |k| k.min(haystacks.len()));
        return (0..n).map(|i| (i, 0)).collect();
    }
    match max {
        Some(0) => Vec::new(),
        Some(k) if k < haystacks.len() => {
            let mut heap: BinaryHeap<Reverse<(u32, usize)>> = BinaryHeap::with_capacity(k + 1);
            for (i, h) in haystacks.iter().enumerate() {
                if let Some(score) = pat.score(h.slice(..), matcher) {
                    if heap.len() < k {
                        heap.push(Reverse((score, i)));
                    } else if heap.peek().map_or(false, |&Reverse((s, _))| score > s) {
                        heap.pop();
                        heap.push(Reverse((score, i)));
                    }
                }
            }
            let mut scored: Vec<(usize, u32)> =
                heap.into_iter().map(|Reverse((s, i))| (i, s)).collect();
            scored.sort_unstable_by(|a, b| b.1.cmp(&a.1));
            scored
        }
        _ => {
            let mut scored: Vec<(usize, u32)> = haystacks
                .iter()
                .enumerate()
                .filter_map(|(i, h)| pat.score(h.slice(..), matcher).map(|s| (i, s)))
                .collect();
            scored.sort_unstable_by(|a, b| b.1.cmp(&a.1));
            scored
        }
    }
}

fn match_with_indices(
    pat: &Pattern,
    items: &[String],
    haystacks: &[Utf32String],
    matcher: &mut Matcher,
    max: Option<usize>,
) -> JsValue {
    let scored: Vec<(usize, u32, Vec<u32>)> = match max {
        Some(0) => Vec::new(),
        Some(k) if k < haystacks.len() => {
            // First pass: score-only top-K via the heap path in `score_all`.
            // Second pass: gather indices for just the K winners.
            let top = score_all(pat, haystacks, matcher, Some(k));
            let mut indices: Vec<u32> = Vec::new();
            top.into_iter()
                .filter_map(|(i, _)| {
                    indices.clear();
                    pat.indices(haystacks[i].slice(..), matcher, &mut indices)
                        .map(|score| {
                            indices.sort_unstable();
                            indices.dedup();
                            (i, score, std::mem::take(&mut indices))
                        })
                })
                .collect()
        }
        _ => {
            let mut acc: Vec<(usize, u32, Vec<u32>)> = Vec::new();
            let mut indices: Vec<u32> = Vec::new();
            for (i, haystack) in haystacks.iter().enumerate() {
                indices.clear();
                if let Some(score) = pat.indices(haystack.slice(..), matcher, &mut indices) {
                    indices.sort_unstable();
                    indices.dedup();
                    acc.push((i, score, std::mem::take(&mut indices)));
                }
            }
            acc.sort_by(|a, b| b.1.cmp(&a.1));
            acc
        }
    };

    let result = js_sys::Array::new();
    for (idx, score, idxs) in scored {
        let triple = js_sys::Array::new();
        triple.push(&JsValue::from_str(&items[idx]));
        triple.push(&JsValue::from_f64(score as f64));
        let js_indices = js_sys::Array::new();
        for i in idxs {
            js_indices.push(&JsValue::from_f64(i as f64));
        }
        triple.push(&js_indices);
        result.push(&triple);
    }
    result.into()
}
