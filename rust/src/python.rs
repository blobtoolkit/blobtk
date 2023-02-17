use pyo3::prelude::*;

// use crate::cli;

mod depth;
mod filter;
mod utils;

#[pymodule]
fn blobtoolkit_core(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    let filter = PyModule::new(py, "filter")?;
    filter.add_function(wrap_pyfunction!(filter::fastx, m)?)?;
    m.add_submodule(filter)?;

    let depth = PyModule::new(py, "depth")?;
    depth.add_function(wrap_pyfunction!(depth::bam_to_bed, m)?)?;
    depth.add_function(wrap_pyfunction!(depth::bam_to_depth, m)?)?;
    // depth.add_class::<bam::DepthOptions>()?;
    m.add_submodule(depth)?;

    Ok(())
}
