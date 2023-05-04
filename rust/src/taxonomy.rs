//!
//! Invoked by calling:
//! `blobtk taxonomy <args>`

use std::collections::HashMap;

use serde;
use serde::{de::Error, Deserialize, Deserializer};

use crate::cli;
use crate::io;

pub use cli::TaxonomyOptions;

#[derive(Clone, Debug)]
struct Name {
    pub tax_id: String,
    pub name: String,
    pub class: Option<String>,
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;

        let mut parts = s.splitn(4, " , ").fuse();

        let tax_id = parts
            .next()
            .ok_or_else(|| D::Error::custom("missing taxon ID"))?
            .into();
        let name = parts
            .next()
            .ok_or_else(|| D::Error::custom("missing name"))?
            .parse()
            .map_err(D::Error::custom)?;
        parts.next();
        let class: String = parts
            .next()
            .ok_or_else(|| D::Error::custom("missing class"))?
            .parse()
            .map_err(D::Error::custom)?;

        let class = if class.is_empty() { Some(class) } else { None };

        Ok(Name {
            tax_id,
            name,
            class,
        })
    }
}

#[derive(Clone, Debug)]
struct Node {
    pub tax_id: String,
    pub parent_id: String,
    pub rank: String,
    pub names: Vec<Name>,
    pub scientific_name: Option<String>,
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;

        let mut parts = s.splitn(4, " , ").fuse();

        let tax_id = parts
            .next()
            .ok_or_else(|| D::Error::custom("missing taxon ID"))?
            .into();
        let parent_id = parts
            .next()
            .ok_or_else(|| D::Error::custom("missing parent ID"))?
            .parse()
            .map_err(D::Error::custom)?;
        let rank = parts
            .next()
            .ok_or_else(|| D::Error::custom("missing rank"))?
            .parse()
            .map_err(D::Error::custom)?;

        let names: Vec<Name> = vec![];

        Ok(Node {
            tax_id,
            parent_id,
            rank,
            names,
            scientific_name: None,
        })
    }
}

/// Execute the `taxonomy` subcommand from `blobtk`.
pub fn taxonomy(options: &cli::TaxonomyOptions) -> Result<(), Box<dyn std::error::Error>> {
    let nodes_file = match options.taxdump.clone() {
        Some(mut d) => {
            d.push("nodes.dmp");
            d
        }
        None => return Ok(()),
    };

    let names_file = match options.taxdump.clone() {
        Some(mut d) => {
            d.push("names.dmp");
            d
        }
        None => return Ok(()),
    };

    let mut nodes = HashMap::new();
    // let mut names = vec![];

    if let Ok(lines) = io::read_lines(nodes_file) {
        for line in lines {
            if let Ok(s) = line {
                let node = serde_json::from_str::<Node>(
                    format!("\"{}\"", s.trim_end_matches("\t|"))
                        .replace("\t|\t", " , ")
                        .replace("\t\t", " , ")
                        .as_str(),
                );
                match node {
                    Ok(n) => {
                        nodes.insert(n.tax_id.clone(), n.clone());
                        println!(
                            "{}, {}, {}",
                            n.names[0].class.as_ref().unwrap(),
                            n.names[0].name,
                            n.names[0].tax_id
                        );
                        println!("{}, {}, {:?}", n.parent_id, n.rank, n.scientific_name);
                        ()
                    }
                    Err(err) => eprintln!("{}", err),
                }
            }
        }
    }
    println!("processed {} nodes", nodes.len());

    if let Ok(lines) = io::read_lines(names_file) {
        for line in lines {
            if let Ok(s) = line {
                let _name = serde_json::from_str::<Name>(
                    format!("\"{}\"", s.trim_end_matches("\t|"))
                        .replace("\t|\t", " , ")
                        .replace("\t\t", " , ")
                        .as_str(),
                );
            }
        }
    }
    println!("processed {} names", nodes.len());
    Ok(())
}
