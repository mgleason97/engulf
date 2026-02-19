use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use serde_json::Value;

/// Options controlling how JSON is transformed into folded-stack lines.
#[derive(Debug, Clone, Default)]
pub struct FlameOpts {
    /// Sequence of keys used to group array elements (objects only).
    /// If empty, every element is treated individually (no grouping).
    pub group_keys: Vec<String>,
}

/// Writer that only counts bytes written; used to measure encoded JSON length.
struct CountWriter {
    bytes: usize,
}

impl CountWriter {
    fn new() -> Self {
        Self { bytes: 0 }
    }
}

impl Write for CountWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.bytes += buf.len();
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Return number of bytes that `v` takes when serialized as JSON.
fn encoded_len(v: &Value) -> usize {
    let mut w = CountWriter::new();
    serde_json::to_writer(&mut w, v).expect("writing to CountWriter never fails");
    w.bytes
}

/// Parse JSON into folded-stack entries sorted by descending weight.
pub fn fold_json_to_stacks(v: &Value, opts: &FlameOpts) -> Vec<(String, u64)> {
    let mut stack: Vec<String> = Vec::new();
    let mut map: HashMap<String, u64> = HashMap::new();

    fn visit(
        val: &Value,
        stack: &mut Vec<String>,
        map: &mut HashMap<String, u64>,
        opts: &FlameOpts,
    ) {
        match val {
            Value::Array(arr) => {
                stack.push("[]".to_string());

                for item in arr {
                    if !opts.group_keys.is_empty() {
                        if let Value::Object(obj) = item {
                            // Push discriminant for the first key that the
                            // object actually contains.
                            if let Some(first_key) =
                                opts.group_keys.iter().find(|k| obj.contains_key(*k))
                            {
                                let discr_val = obj
                                    .get(first_key)
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("<missing>");
                                stack.push(format!("{first_key}={discr_val}"));
                                visit(item, stack, map, opts);
                                stack.pop();
                                continue;
                            }
                        }
                    }

                    // Default behaviour (no grouping or non-object element)
                    visit(item, stack, map, opts);
                }

                stack.pop();
            }
            Value::Object(o) => {
                for (k, v) in o {
                    stack.push(k.clone());
                    visit(v, stack, map, opts);
                    stack.pop();
                }
            }
            Value::String(s) => {
                // Attempt to parse strings that themselves contain JSON so
                // that embedded JSON blobs are also analysed.
                match serde_json::from_str::<Value>(s) {
                    Ok(nested) => visit(&nested, stack, map, opts),
                    Err(_) => {
                        let folded = stack.join(";");
                        *map.entry(folded).or_default() += encoded_len(val) as u64;
                    }
                }
            }
            _ => {
                let folded = stack.join(";");
                *map.entry(folded).or_default() += encoded_len(val) as u64;
            }
        }
    }

    visit(v, &mut stack, &mut map, opts);

    map.into_iter().collect()
}

/// Read JSON from `input`, fold, and write folded stacks.
pub fn write_folded_stacks<R, W>(input: R, mut out: W, opts: &FlameOpts) -> anyhow::Result<()>
where
    R: Read,
    W: Write,
{
    let json: Value = serde_json::from_reader(input)?;
    let stacks = fold_json_to_stacks(&json, opts);
    for (stack, bytes) in stacks {
        writeln!(out, "{stack} {bytes}")?;
    }
    Ok(())
}

/// Read JSON from `path`, fold, and write folded stacks.
pub fn write_folded_stacks_from_file<P: AsRef<Path>, W: Write>(
    path: P,
    out: W,
    opts: &FlameOpts,
) -> anyhow::Result<()> {
    let file = File::open(path)?;
    write_folded_stacks(file, out, opts)
}

#[cfg(test)]
mod tests {
    use super::{FlameOpts, fold_json_to_stacks};
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn folds_simple_object() {
        let v = json!({"a": 1, "b": 2});
        let stacks = fold_json_to_stacks(&v, &FlameOpts::default());
        let map = to_map(stacks);

        assert_eq!(map.get("a"), Some(&1));
        assert_eq!(map.get("b"), Some(&1));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn folds_embedded_json_string() {
        let v = json!({"x": "{\"y\": 1}"});
        let stacks = fold_json_to_stacks(&v, &FlameOpts::default());
        let map = to_map(stacks);

        assert_eq!(map.get("x;y"), Some(&1));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn folds_grouped_array_entries() {
        let v = json!({"items": [{"type": "a", "size": 10}, {"type": "b", "size": 5}]});
        let opts = FlameOpts {
            group_keys: vec!["type".to_string()],
        };
        let stacks = fold_json_to_stacks(&v, &opts);
        let map = to_map(stacks);

        assert_eq!(map.get("items;[];type=a;type"), Some(&json_len("a")));
        assert_eq!(map.get("items;[];type=a;size"), Some(&json_len(10)));
        assert_eq!(map.get("items;[];type=b;type"), Some(&json_len("b")));
        assert_eq!(map.get("items;[];type=b;size"), Some(&json_len(5)));
        assert_eq!(map.len(), 4);
    }

    fn to_map(stacks: Vec<(String, u64)>) -> HashMap<String, u64> {
        stacks.into_iter().collect()
    }

    fn json_len<T: serde::Serialize>(value: T) -> u64 {
        serde_json::to_string(&value).unwrap().len() as u64
    }
}
