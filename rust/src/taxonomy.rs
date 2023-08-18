//!
//! Invoked by calling:
//! `blobtk taxonomy <args>`

use std::collections::HashMap;

use anyhow;

use crate::cli;
use crate::parse;

pub use cli::TaxonomyOptions;

pub use parse::{parse_taxdump, Name, Node};

/// Execute the `taxonomy` subcommand from `blobtk`.
pub fn taxonomy(options: &cli::TaxonomyOptions) -> Result<(), anyhow::Error> {
    let nodes = parse_taxdump(&options.taxdump).unwrap();

    println!("processed {} nodes", nodes.len());

    // TODO: read in taxon names from additonal files
    // TODO: implement hierarcical fuzzy matching

    Ok(())
}
