//!
//! Invoked by calling:
//! `blobtk filter <args>`

use std::error::Error;
use std::io::ErrorKind;

use crate::bam;
use crate::cli;
use crate::fasta;
use crate::fastq;
use crate::io;

pub use cli::FilterOptions;

/// Execute the `filter` subcommand from `blobtk`.
/// Pass a list of sequence names and a BAM file to generate
/// a list of read names and filtered FASTA/FASTQ files.
pub fn filter(options: &cli::FilterOptions) -> Result<(), Box<dyn Error>> {
    let seq_names = io::get_list(&options.list_file);
    if seq_names.is_empty() {
        return Ok(());
    }
    fasta::subsample(
        &seq_names,
        &options.fasta,
        &options.fasta_out,
        &options.suffix,
        &None as &Option<Box<dyn Fn()>>,
    );
    if options.bam.is_none() && options.cram.is_none() {
        return Ok(());
    }
    let bam = bam::open_bam(&options.bam, &options.cram, &options.fasta, true);
    let read_names = bam::reads_from_bam(&seq_names, bam, &None as &Option<Box<dyn Fn()>>);
    fastq::subsample(
        &read_names,
        &options.fastq1,
        &options.fastq2,
        &options.fastq_out,
        &options.suffix,
        &None as &Option<Box<dyn Fn()>>,
    );
    match io::write_list(&read_names, &options.read_list) {
        Err(err) if err.kind() == ErrorKind::BrokenPipe => return Ok(()),
        Err(err) => panic!("unable to write read list file: {}", err),
        Ok(_) => (),
    };
    Ok(())
}
