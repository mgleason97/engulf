use clap::Parser;
use engulf::flamegraph::{FlameOpts, write_folded_stacks_from_file};
use inferno::flamegraph as fgraph;
use std::{io::Cursor, path::PathBuf};

/// engulf – folded stacks from json.
#[derive(Debug, Parser)]
#[command(name = "engulf", version, about)]
struct Cli {
    /// Input JSON file
    input: PathBuf,

    /// Output file
    #[arg(short, long)]
    output: PathBuf,

    /// Group array elements (objects) by one or more keys.
    #[arg(long = "group-by", num_args = 1.., value_name = "KEY")]
    group_by: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let opts = FlameOpts {
        group_keys: cli.group_by.clone(),
    };

    let mut buffer = Vec::new();
    write_folded_stacks_from_file(&cli.input, &mut buffer, &opts)?;

    let writer = open_output(&cli.output)?;
    let mut opts = fgraph::Options::default();
    opts.title = "Foobar".into();

    let reader = Cursor::new(buffer);
    fgraph::from_reader(&mut opts, reader, writer)?;
    Ok(())
}

fn open_output(path: &PathBuf) -> anyhow::Result<Box<dyn std::io::Write>> {
    use std::io::BufWriter;
    Ok(Box::new(BufWriter::new(std::fs::File::create(path)?)))
}
