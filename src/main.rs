use std::io::{self, BufRead, BufWriter, Write};
use std::path::PathBuf;

use clap::Parser;

/// Engulf – a tiny CLI that (eventually) streams data from an input file
/// and optionally writes it to an output destination. For now we only parse
/// and display the provided arguments.
#[derive(Debug, Parser)]
#[command(
    name = "engulf",
    version,
    about = "Stream data from a file and optionally write it elsewhere"
)]
struct Cli {
    /// Path to the input file to read from (required)
    #[arg(value_name = "INPUT")]
    input: PathBuf,

    /// Optional path to write the streamed data to. Defaults to stdout.
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Create a buffered reader for the input file.
    let input_file = std::fs::File::open(&cli.input)?;
    let reader = io::BufReader::new(input_file);

    // Count word frequencies naively.
    use std::collections::HashMap;
    let mut freqs: HashMap<String, usize> = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        for word in line.split_whitespace() {
            *freqs.entry(word.to_string()).or_insert(0) += 1;
        }
    }

    // Setup output destination
    let stdout;
    let mut writer: Box<dyn Write> = if let Some(path) = &cli.output {
        Box::new(BufWriter::new(std::fs::File::create(path)?))
    } else {
        stdout = io::stdout();
        Box::new(BufWriter::new(stdout.lock()))
    };

    // Sort by descending frequency; tie-break alphabetically for stability.
    let mut entries: Vec<_> = freqs.into_iter().collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    for (word, count) in entries {
        writeln!(writer, "{word}: {count}")?;
    }

    writer.flush()?;

    Ok(())
}
