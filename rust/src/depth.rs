//!
//! Invoked by calling:
//! `blobtk depth <args>`

use std::error::Error;

use crate::bam;
use crate::cli;
use crate::io;

pub use bam::BinnedCov;
pub use cli::DepthOptions;

/// Execute the `depth` subcommand from `blobtk`. Generate a BED file.
pub fn depth(options: &cli::DepthOptions) -> Result<(), Box<dyn Error>> {
    let seq_names = io::get_list(&options.list_file);
    let bam = bam::open_bam(&options.bam, &options.cram, &options.fasta, true);
    bam::get_bed_file(bam, &seq_names, options, &None as &Option<Box<dyn Fn()>>);
    Ok(())
}
