//!
//! Invoked by calling:
//! `blobtk plot <args>`

// use std::collections::HashMap;

use crate::blobdir;
use crate::cli;
use crate::snail;
// use crate::io;

pub use cli::PlotOptions;

pub fn plot_snail(meta: &blobdir::Meta, options: &cli::PlotOptions) {
    // let busco_list = meta.busco_list.clone().unwrap();
    let busco_field = meta.busco_list.clone().unwrap()[0].clone();
    let busco_values = blobdir::parse_field_busco(busco_field.0, &options).unwrap();
    let busco_total = busco_field.1;
    let busco_lineage = busco_field.2;
    let gc_values = blobdir::parse_field_float("gc".to_string(), &options).unwrap();
    let length_values = blobdir::parse_field_int("length".to_string(), &options).unwrap();
    let n_values = blobdir::parse_field_float("n".to_string(), &options);
    let ncount_values = blobdir::parse_field_int("ncount".to_string(), &options).unwrap();
    let id = meta.id.clone();
    let record_type = meta.record_type.clone();

    let snail_stats = snail::snail_stats(
        &length_values,
        &gc_values,
        &n_values,
        &ncount_values,
        &busco_values,
        busco_total,
        busco_lineage,
        id,
        record_type,
        &options,
    );
    snail::svg(&snail_stats, &options)

    // let cats = blobdir::parse_field_cat("buscogenes_family".to_string(), &options);
    // let identifiers = blobdir::parse_field_string("identifiers".to_string(), &options);
}

/// Execute the `plot` subcommand from `blobtk`.
pub fn plot(options: &cli::PlotOptions) -> Result<(), Box<dyn std::error::Error>> {
    let meta = blobdir::parse_blobdir(&options);
    let view = &options.view;
    match view {
        Some(cli::View::Snail) => plot_snail(&meta, &options),
        _ => (),
    }
    Ok(())
}