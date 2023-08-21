//!
//! Invoked by calling:
//! `blobtk taxonomy <args>`

use std::collections::HashMap;

use anyhow;

use crate::cli;

/// Functions for ncbi taxonomy processing.
pub mod ncbi;

/// Functions for name lookup.
pub mod lookup;

pub use cli::TaxonomyOptions;

pub use ncbi::{parse_taxdump, write_taxdump};

pub use lookup::build_tries;

/// Execute the `taxonomy` subcommand from `blobtk`.
pub fn taxonomy(options: &cli::TaxonomyOptions) -> Result<(), anyhow::Error> {
    let nodes = parse_taxdump(&options.taxdump).unwrap();
    if let Some(taxdump_out) = options.taxdump_out.clone() {
        let root_taxon_ids = options.root_taxon_id.clone();
        let base_taxon_id = options.base_taxon_id.clone();
        write_taxdump(&nodes, root_taxon_ids, base_taxon_id, taxdump_out);
    }

    let tries = build_tries(&nodes);
    let rank = "genus";
    let higher_rank = "family";
    let trie = tries.get(&format!("{}_{}", rank, higher_rank)).unwrap();
    dbg!(trie.predictive_search(vec!["Arabidopsis", "Brassicaceae"]));
    // TODO: make lookup case insensitive
    // TODO: add support for synonym matching
    // TODO: read in taxon names from additonal files
    // TODO: add support for fuzzy matching?
    // TODO: hang additional taxa on the loaded taxonomy
    Ok(())
}
