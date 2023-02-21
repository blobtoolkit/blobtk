//! `blobtk` is a set of core command line utilities and python bindings for
//! processing common file formats used by [BlobToolKit](https://blobtoolkit.genomehubs.org).

/// Functions for processing BAM files.
pub mod bam;

/// The BlobTk Command Line Interface.
pub mod cli;

/// Summarise windowed coverage depth.
pub mod depth;

/// Functions for processing FASTA files.
pub mod fasta;

/// Functions for processing FASTQ files.
pub mod fastq;

/// Filter files based on a list of sequence IDs.
pub mod filter;

/// Functions for file/terminal IO.
pub mod io;

/// Python bindings.
pub mod python;

/// Utility functions.
pub mod utils;
