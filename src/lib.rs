use nucleo_matcher::pattern::Pattern;
use nucleo_matcher::{Config, Matcher, Utf32Str};
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
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND: &str = r#"
export type MatchResult = [item: string, score: number];
export type MatchResultWithIndices = [item: string, score: number, indices: number[]];
"#;

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
    case_matching: nucleo_matcher::pattern::CaseMatching,
    normalization: nucleo_matcher::pattern::Normalization,
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

        NucleoMatcher {
            matcher: Matcher::new(config),
            items,
            case_matching: opts.case_matching.into(),
            normalization: opts.normalization.into(),
        }
    }

    /// Replace the stored item list.
    #[wasm_bindgen(js_name = "setItems")]
    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
    }

    /// Match a pattern (with fzf-like syntax: `^`, `$`, `'`, `!`) against stored items.
    /// Returns `[item, score]` pairs sorted by score.
    #[wasm_bindgen(js_name = "matchPattern")]
    pub fn match_pattern(&mut self, pattern: &str, options: Option<MatchOptions>) -> JsValue {
        let (cm, norm) = self.resolve_options(&options);
        let str_refs: Vec<&str> = self.items.iter().map(|s| s.as_str()).collect();
        let matches = Pattern::parse(pattern, cm, norm).match_list(&str_refs, &mut self.matcher);
        matches_to_js(matches)
    }

    /// Match a literal pattern against stored items using the specified matching kind.
    /// Special characters are treated literally (no fzf syntax parsing).
    #[wasm_bindgen(js_name = "matchLiteral")]
    pub fn match_literal(&mut self, pattern: &str, kind: Option<AtomKind>, options: Option<MatchOptions>) -> JsValue {
        let (cm, norm) = self.resolve_options(&options);
        let atom_kind: nucleo_matcher::pattern::AtomKind = kind.unwrap_or_default().into();
        let str_refs: Vec<&str> = self.items.iter().map(|s| s.as_str()).collect();
        let matches = Pattern::new(pattern, cm, norm, atom_kind).match_list(&str_refs, &mut self.matcher);
        matches_to_js(matches)
    }

    /// Match a pattern (with fzf-like syntax) and return match indices for highlighting.
    /// Returns `[item, score, indices[]]` triples sorted by score.
    #[wasm_bindgen(js_name = "matchPatternIndices")]
    pub fn match_pattern_indices(&mut self, pattern: &str, options: Option<MatchOptions>) -> JsValue {
        let (cm, norm) = self.resolve_options(&options);
        let pat = Pattern::parse(pattern, cm, norm);
        self.match_with_indices(&pat)
    }

    /// Match a literal pattern and return match indices for highlighting.
    #[wasm_bindgen(js_name = "matchLiteralIndices")]
    pub fn match_literal_indices(&mut self, pattern: &str, kind: Option<AtomKind>, options: Option<MatchOptions>) -> JsValue {
        let (cm, norm) = self.resolve_options(&options);
        let atom_kind: nucleo_matcher::pattern::AtomKind = kind.unwrap_or_default().into();
        let pat = Pattern::new(pattern, cm, norm, atom_kind);
        self.match_with_indices(&pat)
    }

    /// Score a single haystack string against a pattern (with fzf-like syntax).
    /// Returns the score as a number, or `undefined` if no match.
    #[wasm_bindgen]
    pub fn score(&mut self, pattern: &str, haystack: &str, options: Option<MatchOptions>) -> JsValue {
        let (cm, norm) = self.resolve_options(&options);
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
