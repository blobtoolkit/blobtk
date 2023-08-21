//!
//! Invoked by calling:
//! `blobtk taxonomy <args>`

use std::collections::HashMap;

use anyhow;

use crate::cli;

/// Functions for ncbi taxonomy processing.
pub mod ncbi;

pub use cli::TaxonomyOptions;

pub use ncbi::{parse_taxdump, write_taxdump};

/// Execute the `taxonomy` subcommand from `blobtk`.
pub fn taxonomy(options: &cli::TaxonomyOptions) -> Result<(), anyhow::Error> {
    let nodes = parse_taxdump(&options.taxdump).unwrap();
    if let Some(taxdump_out) = options.taxdump_out.clone() {
        let root_taxon_ids = options.root_taxon_id.clone();
        let base_taxon_id = options.base_taxon_id.clone();
        write_taxdump(nodes, root_taxon_ids, base_taxon_id, taxdump_out);
    }

    // println!("processed {} nodes", nodes.nodes.len());

    // TODO: read in taxon names from additonal files
    // TODO: implement hierarcical fuzzy matching

    Ok(())
}
