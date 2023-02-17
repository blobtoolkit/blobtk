use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::bam;
use crate::cli::FilterOptions;
use crate::fasta;
use crate::fastq;
use crate::io;
use crate::python::utils::{
    extract_to_bool, extract_to_default_string, extract_to_option_list, extract_to_option_pathbuf,
};
use pyo3::prelude::*;

#[pymethods]
impl FilterOptions {
    #[new]
    fn new(
        list: Option<HashSet<Vec<u8>>>,
        list_file: Option<PathBuf>,
        bam: Option<PathBuf>,
        cram: Option<PathBuf>,
        fasta: Option<PathBuf>,
        fastq1: Option<PathBuf>,
        fastq2: Option<PathBuf>,
        suffix: String,
        fasta_out: bool,
        fastq_out: bool,
        read_list: Option<PathBuf>,
    ) -> Self {
        FilterOptions {
            list,
            list_file,
            bam,
            cram,
            fasta,
            fastq1,
            fastq2,
            suffix,
            fasta_out,
            fastq_out,
            read_list,
        }
    }
}

#[pyfunction]
pub fn fastx_with_options(options: &FilterOptions, py: Python) -> PyResult<usize> {
    let seq_names = match options.list.to_owned() {
        Some(value) => value,
        _ => match options.list_file.to_owned() {
            value => io::get_list(&value),
        },
    };
    if seq_names.len() == 0 {
        return Ok(0);
    }
    fasta::subsample(
        &seq_names,
        &options.fasta,
        &options.fasta_out,
        &options.suffix,
        &Some(py),
    );
    if options.bam == None && options.cram == None {
        return Ok(0);
    }
    let bam = bam::open_bam(&options.bam, &options.cram, &options.fasta, true);
    let read_names = bam::reads_from_bam(&seq_names, bam, &Some(py));
    io::write_list(&read_names, &options.read_list)?;
    fastq::subsample(
        &read_names,
        &options.fastq1,
        &options.fastq2,
        &options.fastq_out,
        &options.suffix,
        &Some(py),
    );
    Ok(read_names.len())
}

fn convert_hashmap_to_options(py: Python<'_>, map: HashMap<String, PyObject>) -> FilterOptions {
    let list = extract_to_option_list(py, &map, "list");
    let list_file = extract_to_option_pathbuf(py, &map, "list_file");
    let bam = extract_to_option_pathbuf(py, &map, "bam");
    let cram = extract_to_option_pathbuf(py, &map, "cram");
    let fasta = extract_to_option_pathbuf(py, &map, "fasta");
    let fastq1 = extract_to_option_pathbuf(py, &map, "fastq1");
    let fastq2 = extract_to_option_pathbuf(py, &map, "fastq2");
    let read_list = extract_to_option_pathbuf(py, &map, "read_list");
    let suffix = extract_to_default_string(py, &map, "suffix", "filtered");
    let fasta_out = extract_to_bool(py, &map, "fasta_out");
    let fastq_out = extract_to_bool(py, &map, "fastq_out");
    FilterOptions {
        list,
        list_file,
        bam,
        cram,
        fasta,
        fastq1,
        fastq2,
        suffix,
        fasta_out,
        fastq_out,
        read_list,
    }
}

#[pyfunction(kwds = "**")]
pub fn fastx(py: Python<'_>, kwds: Option<HashMap<String, PyObject>>) -> PyResult<usize> {
    let options = match kwds {
        Some(map) => convert_hashmap_to_options(py, map),
        None => panic!["No arguments provided"],
    };
    fastx_with_options(&options, py)
}
