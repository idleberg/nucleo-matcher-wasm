use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct NucleoMatcher {
    matcher: Matcher,
    items: Vec<String>,
}

#[wasm_bindgen]
impl NucleoMatcher {
    #[wasm_bindgen(constructor)]
    pub fn new(items: Vec<JsValue>) -> NucleoMatcher {
        let mut matcher = NucleoMatcher {
            matcher: Matcher::new(Config::DEFAULT),
            items: Vec::new(),
        };
        matcher.set_items(items);
        matcher
    }

    /// Replace the stored item list.
    #[wasm_bindgen(js_name = "setItems")]
    pub fn set_items(&mut self, items: Vec<JsValue>) {
        self.items = items.iter().filter_map(|v| v.as_string()).collect();
    }

    /// Match a pattern against previously stored items. Returns `[item, score]` pairs sorted by score.
    #[wasm_bindgen(js_name = "match")]
    pub fn match_items(&mut self, pattern: &str) -> JsValue {
        let str_refs: Vec<&str> = self.items.iter().map(|s| s.as_str()).collect();

        let matches = Pattern::parse(pattern, CaseMatching::Ignore, Normalization::Smart)
            .match_list(&str_refs, &mut self.matcher);

        let result = js_sys::Array::new();
        for (item, score) in matches {
            let pair = js_sys::Array::new();
            pair.push(&JsValue::from_str(item));
            pair.push(&JsValue::from_f64(score as f64));
            result.push(&pair);
        }
        result.into()
    }

}
