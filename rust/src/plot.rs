//!
//! Invoked by calling:
//! `blobtk plot <args>`

// use std::collections::HashMap;

use crate::blobdir;
use crate::cli;
// use crate::io;

pub use cli::PlotOptions;

/// Execute the `taxonomy` subcommand from `blobtk`.
pub fn plot(options: &cli::PlotOptions) -> Result<(), Box<dyn std::error::Error>> {
    let meta = blobdir::parse_blobdir(&options);
    let lengths = blobdir::parse_field_int("length".to_string(), &options);
    let gcs = blobdir::parse_field_float("gc".to_string(), &options);
    let cats = blobdir::parse_field_cat("buscogenes_family".to_string(), &options);
    let identifiers = blobdir::parse_field_string("identifiers".to_string(), &options);
    let buscos = blobdir::parse_field_busco("lepidoptera_odb10_busco".to_string(), &options);
    Ok(())
}
