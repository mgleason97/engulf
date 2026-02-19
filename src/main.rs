mod flamegraph;

use clap::Parser;
use inferno::flamegraph as fgraph;
use std::{io::Cursor, path::PathBuf};

/// engulf – folded stacks from json.
#[derive(Debug, Parser)]
#[command(name = "engulf", version, about)]
struct Cli {
    /// Input JSON file
    input: PathBuf,

    /// Output file (stdout if omitted)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Group array elements (objects) by one or more keys.
    #[arg(long = "group-by", num_args = 1.., value_name = "KEY")]
    group_by: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    run_flamegraph(&cli)
}

fn open_output(path: &Option<PathBuf>) -> anyhow::Result<Box<dyn std::io::Write>> {
    use std::io::BufWriter;
    if let Some(p) = path {
        Ok(Box::new(BufWriter::new(std::fs::File::create(p)?)))
    } else {
        let stdout = std::io::stdout();
        Ok(Box::new(BufWriter::new(stdout.lock())))
    }
}

fn run_flamegraph(cli: &Cli) -> anyhow::Result<()> {
    use flamegraph::{FlameOpts, write_folded_stacks_from_json_file};

    let opts = FlameOpts {
        group_keys: cli.group_by.clone(),
    };

    let mut buffer = Vec::new();
    write_folded_stacks_from_json_file(&cli.input, &mut buffer, &opts)?;

    let writer = open_output(&cli.output)?;
    let mut opts = fgraph::Options::default();
    opts.title = "Foobar".into();

    let reader = Cursor::new(buffer);
    fgraph::from_reader(&mut opts, reader, writer)?;
    Ok(())
}
