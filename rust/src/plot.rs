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
    let busco_values = blobdir::parse_field_busco("eukaryota_odb10_busco".to_string(), &options);
    let gc_values = blobdir::parse_field_float("gc".to_string(), &options);
    let length_values = blobdir::parse_field_int("length".to_string(), &options);
    let n_values = blobdir::parse_field_float("n".to_string(), &options);
    let ncount_values = blobdir::parse_field_int("ncount".to_string(), &options);

    let snail_stats = snail::snail_stats(
        &length_values,
        &gc_values,
        &n_values,
        &ncount_values,
        &busco_values,
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
