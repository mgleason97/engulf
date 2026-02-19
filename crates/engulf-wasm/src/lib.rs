use std::io::Cursor;

use engulf::flamegraph::{FlameOpts, flamegraph_from_json};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn flamegraph_svg_from_json(
    json: &str,
    title: &str,
    group_keys: Vec<String>,
) -> Result<String, JsValue> {
    let opts = FlameOpts { group_keys };
    let input = Cursor::new(json.as_bytes());
    let mut out = Vec::new();
    flamegraph_from_json(input, &mut out, &opts, title)
        .map_err(|err| JsValue::from_str(&err.to_string()))?;
    String::from_utf8(out).map_err(|err| JsValue::from_str(&err.to_string()))
}
