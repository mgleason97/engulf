use clap::Parser;
use engulf::flamegraph::{FlameOpts, flamegraph_from_json};
use std::io::BufWriter;
use std::{fs::File, path::PathBuf};

/// engulf – create flamegraphs from json.
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

    let input = File::open(&cli.input)?;
    let writer = BufWriter::new(File::create(cli.output)?);
    flamegraph_from_json(input, writer, &opts, "Foobar")?;
    Ok(())
}
