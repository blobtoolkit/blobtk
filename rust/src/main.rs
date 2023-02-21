use std::process;

use blobtk::cli;
use blobtk::depth;
use blobtk::filter;

use std::error::Error;

fn cmd(args: cli::Arguments) -> Result<(), Box<dyn Error>> {
    match args.cmd {
        cli::SubCommand::Filter(options) => filter::filter(&options),
        cli::SubCommand::Depth(options) => depth::depth(&options),
    }
}
fn main() {
    let args = cli::parse();
    if let Err(e) = cmd(args) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
