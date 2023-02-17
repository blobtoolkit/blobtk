use pyo3::prelude::*;

use crate::cli;

mod depth;
mod filter;
mod utils;

#[pymodule]
fn blobtoolkit_core(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    let filter = PyModule::new(py, "filter")?;
    filter.add_function(wrap_pyfunction!(filter::fastx_with_options, m)?)?;
    filter.add_function(wrap_pyfunction!(filter::fastx, m)?)?;
    filter.add_class::<cli::FilterOptions>()?;

    m.add_submodule(filter)?;

    let depth = PyModule::new(py, "depth")?;
    depth.add_function(wrap_pyfunction!(depth::depth_with_options, m)?)?;
    depth.add_function(wrap_pyfunction!(depth::depth, m)?)?;
    depth.add_class::<cli::DepthOptions>()?;

    m.add_submodule(depth)?;

    Ok(())
}
