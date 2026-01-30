use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
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

/// Analyse JSON and return folded-stack entries sorted by descending weight.
pub fn analyze_flamegraph(v: &Value, opts: &FlameOpts) -> Vec<(String, u64)> {
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

/// Convenience helper: read JSON from `path`, analyse, and write folded stacks.
pub fn flamegraph_file(path: &Path, out: &mut dyn Write, opts: &FlameOpts) -> anyhow::Result<()> {
    let file = File::open(path)?;
    let json: Value = serde_json::from_reader(file)?;
    let stacks = analyze_flamegraph(&json, opts);
    for (stack, bytes) in stacks {
        writeln!(out, "{stack} {bytes}")?;
    }
    Ok(())
}
