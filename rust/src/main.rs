use std::process;

use blobtk::cli;
use blobtk::run;

fn main() {
    let args = cli::parse();
    if let Err(e) = run::cmd(args) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
