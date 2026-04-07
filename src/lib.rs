use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use wasm_bindgen::prelude::*;

fn parse_case_matching(val: &JsValue) -> CaseMatching {
    match val.as_string().as_deref() {
        Some("smart") => CaseMatching::Smart,
        Some("respect") => CaseMatching::Respect,
        _ => CaseMatching::Ignore,
    }
}

fn parse_normalization(val: &JsValue) -> Normalization {
    match val.as_string().as_deref() {
        Some("never") => Normalization::Never,
        _ => Normalization::Smart,
    }
}

fn parse_atom_kind(kind: Option<String>) -> AtomKind {
    match kind.as_deref() {
        Some("substring") => AtomKind::Substring,
        Some("prefix") => AtomKind::Prefix,
        Some("postfix") => AtomKind::Postfix,
        Some("exact") => AtomKind::Exact,
        _ => AtomKind::Fuzzy,
    }
}

fn resolve_case_matching(options: &JsValue, default: CaseMatching) -> CaseMatching {
    if options.is_object() {
        let val = js_sys::Reflect::get(options, &JsValue::from_str("caseMatching")).unwrap_or(JsValue::UNDEFINED);
        if !val.is_undefined() {
            return parse_case_matching(&val);
        }
    }
    default
}

fn resolve_normalization(options: &JsValue, default: Normalization) -> Normalization {
    if options.is_object() {
        let val = js_sys::Reflect::get(options, &JsValue::from_str("normalization")).unwrap_or(JsValue::UNDEFINED);
        if !val.is_undefined() {
            return parse_normalization(&val);
        }
    }
    default
}

fn matches_to_js<S: AsRef<str>>(matches: Vec<(S, u32)>) -> JsValue {
    let result = js_sys::Array::new();
    for (item, score) in matches {
        let pair = js_sys::Array::new();
        pair.push(&JsValue::from_str(item.as_ref()));
        pair.push(&JsValue::from_f64(score as f64));
        result.push(&pair);
    }
    result.into()
}

#[wasm_bindgen]
pub struct NucleoMatcher {
    matcher: Matcher,
    items: Vec<String>,
    case_matching: CaseMatching,
    normalization: Normalization,
}

#[wasm_bindgen]
impl NucleoMatcher {
    /// Create a new matcher with items and optional configuration.
    ///
    /// Options:
    /// - `matchPaths` (boolean) — treat `/` and `\` as word boundaries (default: false)
    /// - `preferPrefix` (boolean) — boost matches near the start (default: false)
    /// - `caseMatching` ("ignore" | "smart" | "respect") — case sensitivity (default: "ignore")
    /// - `normalization` ("smart" | "never") — unicode normalization (default: "smart")
    #[wasm_bindgen(constructor)]
    pub fn new(items: Vec<JsValue>, options: JsValue) -> NucleoMatcher {
        let mut config = Config::DEFAULT;
        let mut case_matching = CaseMatching::Ignore;
        let mut normalization = Normalization::Smart;

        if options.is_object() {
            if let Ok(val) = js_sys::Reflect::get(&options, &JsValue::from_str("matchPaths")) {
                if val.as_bool() == Some(true) {
                    config = config.match_paths();
                }
            }
            if let Ok(val) = js_sys::Reflect::get(&options, &JsValue::from_str("preferPrefix")) {
                if val.as_bool() == Some(true) {
                    config.prefer_prefix = true;
                }
            }
            if let Ok(val) = js_sys::Reflect::get(&options, &JsValue::from_str("caseMatching")) {
                if !val.is_undefined() {
                    case_matching = parse_case_matching(&val);
                }
            }
            if let Ok(val) = js_sys::Reflect::get(&options, &JsValue::from_str("normalization")) {
                if !val.is_undefined() {
                    normalization = parse_normalization(&val);
                }
            }
        }

        let stored_items: Vec<String> = items.iter().filter_map(|v| v.as_string()).collect();

        NucleoMatcher {
            matcher: Matcher::new(config),
            items: stored_items,
            case_matching,
            normalization,
        }
    }

    /// Replace the stored item list.
    #[wasm_bindgen(js_name = "setItems")]
    pub fn set_items(&mut self, items: Vec<JsValue>) {
        self.items = items.iter().filter_map(|v| v.as_string()).collect();
    }

    /// Match a pattern (with fzf-like syntax: `^`, `$`, `'`, `!`) against stored items.
    /// Returns `[item, score]` pairs sorted by score.
    /// Per-call `options` can override `caseMatching` and `normalization`.
    #[wasm_bindgen(js_name = "matchPattern")]
    pub fn match_pattern(&mut self, pattern: &str, options: JsValue) -> JsValue {
        let cm = resolve_case_matching(&options, self.case_matching);
        let norm = resolve_normalization(&options, self.normalization);
        let str_refs: Vec<&str> = self.items.iter().map(|s| s.as_str()).collect();
        let matches = Pattern::parse(pattern, cm, norm).match_list(&str_refs, &mut self.matcher);
        matches_to_js(matches)
    }

    /// Match a literal pattern against stored items using the specified matching kind.
    /// Special characters are treated literally (no fzf syntax parsing).
    /// `kind`: "fuzzy" (default), "substring", "prefix", "postfix", "exact".
    /// Returns `[item, score]` pairs sorted by score.
    #[wasm_bindgen(js_name = "matchLiteral")]
    pub fn match_literal(&mut self, pattern: &str, kind: Option<String>, options: JsValue) -> JsValue {
        let cm = resolve_case_matching(&options, self.case_matching);
        let norm = resolve_normalization(&options, self.normalization);
        let atom_kind = parse_atom_kind(kind);
        let str_refs: Vec<&str> = self.items.iter().map(|s| s.as_str()).collect();
        let matches = Pattern::new(pattern, cm, norm, atom_kind).match_list(&str_refs, &mut self.matcher);
        matches_to_js(matches)
    }

    /// Match a pattern (with fzf-like syntax) and return match indices for highlighting.
    /// Returns `[item, score, indices[]]` triples sorted by score.
    #[wasm_bindgen(js_name = "matchPatternIndices")]
    pub fn match_pattern_indices(&mut self, pattern: &str, options: JsValue) -> JsValue {
        let cm = resolve_case_matching(&options, self.case_matching);
        let norm = resolve_normalization(&options, self.normalization);
        let pat = Pattern::parse(pattern, cm, norm);
        self.match_with_indices(&pat)
    }

    /// Match a literal pattern and return match indices for highlighting.
    /// `kind`: "fuzzy" (default), "substring", "prefix", "postfix", "exact".
    /// Returns `[item, score, indices[]]` triples sorted by score.
    #[wasm_bindgen(js_name = "matchLiteralIndices")]
    pub fn match_literal_indices(&mut self, pattern: &str, kind: Option<String>, options: JsValue) -> JsValue {
        let cm = resolve_case_matching(&options, self.case_matching);
        let norm = resolve_normalization(&options, self.normalization);
        let atom_kind = parse_atom_kind(kind);
        let pat = Pattern::new(pattern, cm, norm, atom_kind);
        self.match_with_indices(&pat)
    }

    /// Score a single haystack string against a pattern (with fzf-like syntax).
    /// Returns the score as a number, or `undefined` if no match.
    #[wasm_bindgen]
    pub fn score(&mut self, pattern: &str, haystack: &str, options: JsValue) -> JsValue {
        let cm = resolve_case_matching(&options, self.case_matching);
        let norm = resolve_normalization(&options, self.normalization);
        let pat = Pattern::parse(pattern, cm, norm);
        let mut buf = Vec::new();
        let haystack_utf32 = Utf32Str::new(haystack, &mut buf);
        match pat.score(haystack_utf32, &mut self.matcher) {
            Some(score) => JsValue::from_f64(score as f64),
            None => JsValue::UNDEFINED,
        }
    }
}

impl NucleoMatcher {
    fn match_with_indices(&mut self, pat: &Pattern) -> JsValue {
        let mut scored: Vec<(&str, u32, Vec<u32>)> = Vec::new();
        let mut indices = Vec::new();
        let mut buf = Vec::new();

        for item in &self.items {
            indices.clear();
            let haystack = Utf32Str::new(item.as_str(), &mut buf);
            if let Some(score) = pat.indices(haystack, &mut self.matcher, &mut indices) {
                indices.sort_unstable();
                indices.dedup();
                scored.push((item.as_str(), score, indices.clone()));
            }
        }

        scored.sort_by(|a, b| b.1.cmp(&a.1));

        let result = js_sys::Array::new();
        for (item, score, idxs) in scored {
            let triple = js_sys::Array::new();
            triple.push(&JsValue::from_str(item));
            triple.push(&JsValue::from_f64(score as f64));
            let js_indices = js_sys::Array::new();
            for idx in idxs {
                js_indices.push(&JsValue::from_f64(idx as f64));
            }
            triple.push(&js_indices);
            result.push(&triple);
        }
        result.into()
    }
}
