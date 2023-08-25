use std::collections::HashSet;

use trie_rs::{Trie, TrieBuilder};

use crate::{taxonomy::parse, utils::styled_progress_bar};

use parse::Nodes;

pub fn build_trie(nodes: &Nodes) -> Trie<String> {
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
    let mut builder = TrieBuilder::<String>::new();

    let rank_set: HashSet<&str> = HashSet::from_iter(ranks.iter().cloned());
    let higher_rank_set: HashSet<&str> = HashSet::from_iter(higher_ranks.iter().cloned());
    let node_count = nodes.nodes.len();
    let progress_bar = styled_progress_bar(node_count, "Building lookup trie");

    for (tax_id, node) in nodes.nodes.iter() {
        progress_bar.inc(1);
        if rank_set.contains(node.rank.as_str()) {
            let lineage = nodes.lineage(&"1".to_string(), tax_id);
            for n in lineage.into_iter().rev() {
                if higher_rank_set.contains(n.rank.as_str()) {
                    builder.push(vec![
                        node.rank(),
                        node.lc_scientific_name(),
                        n.rank(),
                        n.lc_scientific_name(),
                        node.tax_id(),
                    ])
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
    eprintln!("Building trie of {} nodes...", node_count);
    let trie = builder.build();
    eprintln!("    ...done");
    trie
}

pub fn lookup_nodes(new_nodes: &Nodes, nodes: &Nodes) {
    let trie = build_trie(&nodes);
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
            let mut tax_id = None;
            for n in lineage.into_iter().rev() {
                if higher_rank_set.contains(n.rank.as_str()) {
                    let matches = trie.predictive_search(vec![
                        node.rank(),
                        node.lc_scientific_name(),
                        n.rank(),
                        n.lc_scientific_name(),
                    ]);
                    if matches.len() == 1 {
                        tax_id = Some(matches[0][4].clone());
                        continue;
                    }
                    // if let Some(names) = n.names.as_ref() {
                    //     for name in names {
                    //         builder.push(name.name.split(" ").collect::<Vec<&str>>());
                    //     }
                    // }
                }
            }
            if let Some(ref_tax_id) = tax_id {
                hits.push(ref_tax_id.clone());
                // eprintln!("{}: {}", ref_tax_id, node.tax_id());
                continue;
            }
        }
    }
    progress_bar.finish();
    dbg!(hits.len());
}
