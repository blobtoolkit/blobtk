use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use crate::{taxonomy::parse, utils::styled_progress_bar};

use parse::Nodes;

pub fn build_lookup(nodes: &Nodes) -> HashMap<String, Vec<String>> {
    let ranks = [
        "subspecies",
        "species",
        "genus",
        "family",
        "order",
        "class",
        "phylum",
        "kingdom",
    ];
    let higher_ranks = ["family", "order", "class", "phylum", "kingdom"];
    let mut table = HashMap::new();

    let rank_set: HashSet<&str> = HashSet::from_iter(ranks.iter().cloned());
    let higher_rank_set: HashSet<&str> = HashSet::from_iter(higher_ranks.iter().cloned());
    let node_count = nodes.nodes.len();
    let progress_bar = styled_progress_bar(node_count, "Building lookup hash");

    for (tax_id, node) in nodes.nodes.iter() {
        progress_bar.inc(1);
        if rank_set.contains(node.rank.as_str()) {
            let lineage = nodes.lineage(&"1".to_string(), tax_id);
            for n in lineage.into_iter().rev() {
                if higher_rank_set.contains(n.rank.as_str()) {
                    let key = format!(
                        "{}:{}:{}:{}",
                        node.rank_letter(),
                        node.lc_scientific_name(),
                        n.rank_letter(),
                        n.lc_scientific_name()
                    );
                    match table.entry(key) {
                        Entry::Vacant(e) => {
                            e.insert(vec![node.tax_id()]);
                        }
                        Entry::Occupied(mut e) => {
                            e.get_mut().push(node.tax_id());
                        }
                    }
                    // if let Some(names) = n.names.as_ref() {
                    //     for name in names {
                    //         builder.push(name.name.split(" ").collect::<Vec<&str>>());
                    //     }
                    // }
                }
            }
        }
    }
    progress_bar.finish();
    table
}

pub fn lookup_nodes(new_nodes: &Nodes, nodes: &Nodes) {
    let table = build_lookup(&nodes);
    let ranks = [
        "subspecies",
        "species",
        "genus",
        "family",
        "order",
        "class",
        "phylum",
        "kingdom",
    ];
    let higher_ranks = ["family", "order", "class", "phylum", "kingdom"];
    let rank_set: HashSet<&str> = HashSet::from_iter(ranks.iter().cloned());
    let higher_rank_set: HashSet<&str> = HashSet::from_iter(higher_ranks.iter().cloned());
    let node_count = new_nodes.nodes.len();
    let progress_bar = styled_progress_bar(node_count, "Looking up names");
    let mut hits = vec![];

    for (tax_id, node) in new_nodes.nodes.iter() {
        progress_bar.inc(1);
        if rank_set.contains(node.rank.as_str()) {
            let lineage = new_nodes.lineage(&"1".to_string(), tax_id);
            let mut match_tax_id = None;
            for n in lineage.into_iter().rev() {
                if higher_rank_set.contains(n.rank.as_str()) {
                    let key = format!(
                        "{}:{}:{}:{}",
                        node.rank_letter(),
                        node.lc_scientific_name(),
                        n.rank_letter(),
                        n.lc_scientific_name()
                    );
                    match table.get(&key) {
                        None => (),
                        Some(value) => {
                            if value.len() == 1 {
                                match_tax_id = Some(value[0].clone());
                            }
                        }
                    };
                }
            }
            if let Some(ref_tax_id) = match_tax_id {
                hits.push(ref_tax_id.clone());
                // eprintln!("{}: {}", ref_tax_id, node.tax_id());
                continue;
            }
        }
    }
    progress_bar.finish();
    dbg!(hits.len());
}
