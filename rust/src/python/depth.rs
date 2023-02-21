use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::bam::{self, BinnedCov};
use crate::cli::DepthOptions;
use crate::io;
use crate::python::utils::{extract_to_option_list, extract_to_option_pathbuf, extract_to_usize};
use pyo3::prelude::*;

#[pymethods]
impl DepthOptions {
    #[new]
    fn new(
        bin_size: usize,
        list: Option<HashSet<Vec<u8>>>,
        list_file: Option<PathBuf>,
        bam: Option<PathBuf>,
        cram: Option<PathBuf>,
        fasta: Option<PathBuf>,
        bed: Option<PathBuf>,
    ) -> Self {
        DepthOptions {
            list,
            list_file,
            bam,
            cram,
            fasta,
            bin_size,
            bed,
        }
    }
}

#[pyfunction]
pub fn bam_to_bed_with_options(options: &DepthOptions, py: Python) -> PyResult<usize> {
    let seq_names = match options.list.to_owned() {
        Some(value) => value,
        _ => {
            let value = options.list_file.to_owned();
            io::get_list(&value)
        }
    };
    let ctrlc_wrapper = || {
        py.check_signals().unwrap();
    };
    let bam = bam::open_bam(&options.bam, &options.cram, &options.fasta, true);
    bam::get_bed_file(bam, &seq_names, options, &Some(Box::new(ctrlc_wrapper)));
    Ok(1)
}

#[pyfunction]
pub fn bam_to_depth_with_options(options: &DepthOptions, py: Python) -> Vec<BinnedCov> {
    let seq_names = match options.list.to_owned() {
        Some(value) => value,
        _ => {
            let value = options.list_file.to_owned();
            io::get_list(&value)
        }
    };
    let ctrlc_wrapper = || {
        py.check_signals().unwrap();
    };
    let bam = bam::open_bam(&options.bam, &options.cram, &options.fasta, true);
    bam::get_depth(bam, &seq_names, options, &Some(Box::new(ctrlc_wrapper)))
}

fn convert_hashmap_to_options(py: Python<'_>, map: HashMap<String, PyObject>) -> DepthOptions {
    let list = extract_to_option_list(py, &map, "list");
    let list_file = extract_to_option_pathbuf(py, &map, "list_file");
    let bam = extract_to_option_pathbuf(py, &map, "bam");
    let cram = extract_to_option_pathbuf(py, &map, "cram");
    let fasta = extract_to_option_pathbuf(py, &map, "fasta");
    let bed = extract_to_option_pathbuf(py, &map, "bed");
    let bin_size = extract_to_usize(py, &map, "bin_size");
    DepthOptions {
        bin_size,
        list,
        list_file,
        bam,
        cram,
        fasta,
        bed,
    }
}

#[pyfunction]
#[pyo3(signature = (**kwds))]
pub fn bam_to_bed(py: Python<'_>, kwds: Option<HashMap<String, PyObject>>) -> PyResult<()> {
    let options = match kwds {
        Some(map) => convert_hashmap_to_options(py, map),
        None => panic!["No arguments provided"],
    };
    bam_to_bed_with_options(&options, py)?;
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (**kwds))]
pub fn bam_to_depth(py: Python<'_>, kwds: Option<HashMap<String, PyObject>>) -> Vec<BinnedCov> {
    let options = match kwds {
        Some(map) => convert_hashmap_to_options(py, map),
        None => panic!["No arguments provided"],
    };
    bam_to_depth_with_options(&options, py)
}
