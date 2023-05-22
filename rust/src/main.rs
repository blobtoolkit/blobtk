use std::process;

use anyhow;

use blobtk::cli;
use blobtk::depth;
use blobtk::filter;
use blobtk::plot;
use blobtk::taxonomy;

fn cmd(args: cli::Arguments) -> Result<(), anyhow::Error> {
    match args.cmd {
        cli::SubCommand::Filter(options) => filter::filter(&options)?,
        cli::SubCommand::Depth(options) => depth::depth(&options)?,
        cli::SubCommand::Plot(options) => plot::plot(&options)?,
        cli::SubCommand::Taxonomy(options) => taxonomy::taxonomy(&options)?,
    }
    Ok(())
}
fn main() {
    let args = cli::parse();
    if let Err(e) = cmd(args) {
        eprintln!("ERROR: {e}");
        process::exit(1);
    }
}
