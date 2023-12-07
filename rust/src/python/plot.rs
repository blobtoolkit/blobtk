use std::collections::HashMap;

use crate::blobdir::parse_blobdir;
use crate::cli::{self, PlotOptions, View};
use crate::plot::{plot_blob, plot_cumulative, plot_legend, plot_snail};
use crate::python::utils::{
    extract_to_option_f64, extract_to_option_origin, extract_to_option_palette,
    extract_to_option_reducer, extract_to_option_scale, extract_to_option_showlegend,
    extract_to_option_string, extract_to_option_usize, extract_to_option_vec_string,
    extract_to_pathbuf, extract_to_view,
};
use pyo3::prelude::*;

// #[pymethods]
// impl PlotOptions {
//     #[new]
//     #[allow(clippy::too_many_arguments)]
//     fn new(
//         blobdir: PathBuf,
//         view: View,
//         output: Option<String>,
//         filter: Option<Vec<String>>,
//         segments: Option<usize>,
//         max_span: Option<usize>,
//         max_scaffold: Option<usize>,
//         x_field: Option<String>,
//         y_field: Option<String>,
//         z_field: Option<String>,
//         cat_field: Option<String>,
//         resolution: Option<usize>,
//         hist_height: Option<usize>,
//         reducer_function: Option<Reducer>,
//         scale_function: Option<Scale>,
//         scale_factor: Option<f64>,
//         x_limit: Option<String>,
//         y_limit: Option<String>,
//         cat_count: Option<usize>,
//         show_legend: Option<ShowLegend>,
//         cat_order: Option<String>,
//         origin: Option<Origin>,
//         palette: Option<Palette>,
//         color: Option<Vec<String>>,
//     ) -> Self {
//         PlotOptions {
//             blobdir,
//             view,
//             output: output.unwrap_or(String::from("output.svg")),
//             filter: filter.unwrap_or_default(),
//             segments: segments.unwrap_or(1000),
//             max_span,
//             max_scaffold,
//             x_field,
//             y_field,
//             z_field,
//             cat_field,
//             resolution: resolution.unwrap_or(30),
//             hist_height,
//             reducer_function: reducer_function.unwrap_or_default(),
//             scale_function: scale_function.unwrap_or_default(),
//             scale_factor: scale_factor.unwrap_or(1.0),
//             x_limit,
//             y_limit,
//             cat_count: cat_count.unwrap_or(10),
//             show_legend: show_legend.unwrap_or_default(),
//             cat_order,
//             origin,
//             palette,
//             color,
//         }
//     }
// }

#[pyfunction]
pub fn plot_with_options(options: &PlotOptions) -> PyResult<()> {
    let meta = parse_blobdir(&options.blobdir).unwrap();
    let view = &options.view;
    match view {
        cli::View::Blob => plot_blob(&meta, &options).unwrap(),
        cli::View::Cumulative => plot_cumulative(&meta, &options).unwrap(),
        cli::View::Legend => plot_legend(&meta, &options).unwrap(),
        cli::View::Snail => plot_snail(&meta, &options).unwrap(),
    }
    Ok(())
}

fn convert_hashmap_to_options(py: Python<'_>, map: HashMap<String, PyObject>) -> PlotOptions {
    let blobdir = extract_to_pathbuf(py, &map, "blobdir");
    let view = extract_to_view(py, &map, "view");
    let output = extract_to_option_string(py, &map, "output");
    let filter = extract_to_option_vec_string(py, &map, "filter");
    let segments = extract_to_option_usize(py, &map, "segments");
    let max_span = extract_to_option_usize(py, &map, "max_span");
    let max_scaffold = extract_to_option_usize(py, &map, "max_scaffold");
    let x_field = extract_to_option_string(py, &map, "x_field");
    let y_field = extract_to_option_string(py, &map, "y_field");
    let z_field = extract_to_option_string(py, &map, "z_field");
    let cat_field = extract_to_option_string(py, &map, "cat_field");
    let resolution = extract_to_option_usize(py, &map, "resolution");
    let hist_height = extract_to_option_usize(py, &map, "hist_height");
    let reducer_function = extract_to_option_reducer(py, &map, "reducer_function");
    let scale_function = extract_to_option_scale(py, &map, "scale_function");
    let scale_factor = extract_to_option_f64(py, &map, "scale_factor");
    let x_limit = extract_to_option_string(py, &map, "x_limit");
    let y_limit = extract_to_option_string(py, &map, "y_limit");
    let cat_count = extract_to_option_usize(py, &map, "cat_count");
    let show_legend = extract_to_option_showlegend(py, &map, "show_legend");
    let cat_order = extract_to_option_string(py, &map, "cat_order");
    let origin = extract_to_option_origin(py, &map, "origin");
    let palette = extract_to_option_palette(py, &map, "palette");
    let color = extract_to_option_vec_string(py, &map, "color");
    PlotOptions {
        blobdir,
        view,
        output: output.unwrap_or(String::from("output.svg")),
        filter: filter.unwrap_or_default(),
        segments: segments.unwrap_or(1000),
        max_span,
        max_scaffold,
        x_field,
        y_field,
        z_field,
        cat_field,
        resolution: resolution.unwrap_or(30),
        hist_height,
        reducer_function: reducer_function.unwrap_or_default(),
        scale_function: scale_function.unwrap_or_default(),
        scale_factor: scale_factor.unwrap_or(1.0),
        x_limit,
        y_limit,
        cat_count: cat_count.unwrap_or(10),
        show_legend: show_legend.unwrap_or_default(),
        cat_order,
        origin,
        palette,
        color,
    }
}

#[pyfunction]
#[pyo3(signature = (**kwds))]
pub fn plot(py: Python<'_>, kwds: Option<HashMap<String, PyObject>>) -> PyResult<()> {
    let options = match kwds {
        Some(map) => convert_hashmap_to_options(py, map),
        None => panic!["No arguments provided"],
    };
    plot_with_options(&options)
}

#[pyfunction]
#[pyo3(signature = (**kwds))]
pub fn blob(py: Python<'_>, kwds: Option<HashMap<String, PyObject>>) -> PyResult<()> {
    let options = match kwds {
        Some(map) => convert_hashmap_to_options(py, map),
        None => panic!["No arguments provided"],
    };
    let blob_options = PlotOptions {
        view: View::Blob,
        ..options
    };
    plot_with_options(&blob_options)
}

#[pyfunction]
#[pyo3(signature = (**kwds))]
pub fn cumulative(py: Python<'_>, kwds: Option<HashMap<String, PyObject>>) -> PyResult<()> {
    let options = match kwds {
        Some(map) => convert_hashmap_to_options(py, map),
        None => panic!["No arguments provided"],
    };
    let cumulative_options = PlotOptions {
        view: View::Cumulative,
        ..options
    };
    plot_with_options(&cumulative_options)
}

#[pyfunction]
#[pyo3(signature = (**kwds))]
pub fn legend(py: Python<'_>, kwds: Option<HashMap<String, PyObject>>) -> PyResult<()> {
    let options = match kwds {
        Some(map) => convert_hashmap_to_options(py, map),
        None => panic!["No arguments provided"],
    };
    let legend_options = PlotOptions {
        view: View::Legend,
        ..options
    };
    plot_with_options(&legend_options)
}

#[pyfunction]
#[pyo3(signature = (**kwds))]
pub fn snail(py: Python<'_>, kwds: Option<HashMap<String, PyObject>>) -> PyResult<()> {
    let options = match kwds {
        Some(map) => convert_hashmap_to_options(py, map),
        None => panic!["No arguments provided"],
    };
    let snail_options = PlotOptions {
        view: View::Snail,
        ..options
    };
    plot_with_options(&snail_options)
}
