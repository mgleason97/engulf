use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

use serde_json::Value;

/// Strategy for grouping array elements when building folded stacks.
#[derive(Debug, Clone)]
pub enum ArrayGrouping {
    /// treat every element individually
    None,
    /// group array elements of object type by the value of a given key
    Key(String),
}

#[derive(Debug, Clone)]
pub struct FlameOpts {
    pub grouping: ArrayGrouping,
}

impl Default for FlameOpts {
    fn default() -> Self {
        Self {
            grouping: ArrayGrouping::None,
        }
    }
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
                    match (&opts.grouping, item) {
                        (ArrayGrouping::Key(k), Value::Object(obj)) if obj.get(k).is_some() => {
                            let discr = format!("{}={}", k, obj[k].as_str().unwrap_or("<non-str>"));
                            stack.push(discr);
                            visit(item, stack, map, opts);
                            stack.pop();
                        }
                        _ => visit(item, stack, map, opts),
                    }
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
            Value::String(s) => match serde_json::from_str::<Value>(s) {
                Ok(v) => visit(&v, stack, map, opts),
                Err(_) => {
                    let folded = stack.join(";");
                    *map.entry(folded).or_default() += encoded_len(val) as u64;
                }
            },
            _ => {
                let folded = stack.join(";");
                *map.entry(folded).or_default() += encoded_len(val) as u64;
            }
        }
    }

    visit(v, &mut stack, &mut map, opts);

    let mut entries: Vec<_> = map.into_iter().collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    entries
}

/// Convenience helper: read JSON from `path`, analyze, and write folded stacks.
pub fn flamegraph_file(path: &Path, out: &mut dyn Write, opts: &FlameOpts) -> anyhow::Result<()> {
    let file = File::open(path)?;
    let json: Value = serde_json::from_reader(file)?;
    let stacks = analyze_flamegraph(&json, opts);
    for (stack, bytes) in stacks {
        writeln!(out, "{stack} {bytes}")?;
    }
    Ok(())
}
