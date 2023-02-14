use std::error::Error;
use std::io::ErrorKind;

use crate::bam;
use crate::cli;
use crate::fasta;
use crate::fastq;
use crate::io;

pub fn filter(options: &cli::FilterOptions) -> Result<(), Box<dyn Error>> {
    let seq_names = io::get_list(&options.list_file);
    if seq_names.len() == 0 {
        return Ok(());
    }
    fasta::subsample(
        &seq_names,
        &options.fasta,
        &options.fasta_out,
        &options.suffix,
    );
    if options.bam == None && options.cram == None {
        return Ok(());
    }
    let bam = bam::open_bam(&options.bam, &options.cram, &options.fasta, true);
    let read_names = bam::reads_from_bam(&seq_names, bam);
    fastq::subsample(
        &read_names,
        &options.fastq1,
        &options.fastq2,
        &options.fastq_out,
        &options.suffix,
    );
    match io::write_list(&read_names, &options.read_list) {
        Err(err) if err.kind() == ErrorKind::BrokenPipe => return Ok(()),
        Err(err) => panic!("unable to write read list file: {}", err),
        Ok(_) => (),
    };
    Ok(())
}

pub fn depth(options: &cli::DepthOptions) -> Result<(), Box<dyn Error>> {
    let seq_names = io::get_list(&options.list_file);
    let bam = bam::open_bam(&options.bam, &options.cram, &options.fasta, true);
    bam::get_depth(bam, &seq_names, &options);
    Ok(())
}

pub fn cmd(args: cli::Arguments) -> Result<(), Box<dyn Error>> {
    match args.cmd {
        cli::SubCommand::Filter(options) => filter(&options),
        cli::SubCommand::Depth(options) => depth(&options),
    }
}
