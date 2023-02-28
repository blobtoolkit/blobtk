use std::process;

use blobtk::cli;
use blobtk::depth;
use blobtk::filter;
use blobtk::taxonomy;

use std::error::Error;

fn cmd(args: cli::Arguments) -> Result<(), Box<dyn Error>> {
    match args.cmd {
        cli::SubCommand::Filter(options) => filter::filter(&options),
        cli::SubCommand::Depth(options) => depth::depth(&options),
        cli::SubCommand::Taxonomy(options) => taxonomy::taxonomy(&options),
    }
}
fn main() {
    let args = cli::parse();
    if let Err(e) = cmd(args) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
