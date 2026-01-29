mod flamegraph;

use clap::Parser;
use std::path::PathBuf;

/// Engulf – analyse JSON and emit results.
#[derive(Debug, Parser)]
#[command(name = "engulf", version, about)]
struct Cli {
    /// Input JSON file
    input: PathBuf,

    /// Output file (stdout if omitted)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Group array elements by value of this key (flamegraph mode)
    #[arg(long = "group-by")]
    group_by: Option<String>,
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
    use flamegraph::{ArrayGrouping, FlameOpts, flamegraph_file};

    let opts = FlameOpts {
        grouping: cli
            .group_by
            .as_ref()
            .map(|k| ArrayGrouping::Key(k.clone()))
            .unwrap_or(ArrayGrouping::None),
    };

    let mut writer = open_output(&cli.output)?;
    flamegraph_file(&cli.input, &mut writer, &opts)
}
