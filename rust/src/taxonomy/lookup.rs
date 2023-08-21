use std::collections::{HashMap, HashSet};

use trie_rs::{Trie, TrieBuilder};

use crate::taxonomy::ncbi;

use ncbi::Nodes;

pub fn build_tries(nodes: &Nodes) -> HashMap<String, Trie<&str>> {
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
    let mut builders = HashMap::new();
    for rank in ranks {
        for higher_rank in higher_ranks {
            builders.insert(
                format!("{}_{}", rank, higher_rank),
                TrieBuilder::<&str>::new(),
            );
        }
    }

    let rank_set: HashSet<&str> = HashSet::from_iter(ranks.iter().cloned());
    let higher_rank_set: HashSet<&str> = HashSet::from_iter(higher_ranks.iter().cloned());
    for (tax_id, node) in nodes.nodes.iter() {
        if rank_set.contains(node.rank.as_str()) {
            let mut lineage = nodes.lineage(&"1".to_string(), tax_id);
            for n in lineage.into_iter().rev() {
                if higher_rank_set.contains(n.rank.as_str()) {
                    let builder = builders
                        .get_mut(&format!("{}_{}", node.rank, n.rank))
                        .unwrap();
                    builder.push(vec![
                        node.scientific_name.as_ref().unwrap().as_str(),
                        n.scientific_name.as_ref().unwrap().as_str(),
                        node.tax_id.as_str(),
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
    let mut tries = HashMap::new();
    for rank in ranks {
        for higher_rank in higher_ranks {
            let builder = builders
                .get_mut(&format!("{}_{}", rank, higher_rank))
                .unwrap();
            let trie = builder.build();
            tries.insert(format!("{}_{}", rank, higher_rank), trie);
        }
    }
    tries
}
