//!
//! Invoked by calling:
//! `blobtk taxonomy <args>`

use serde;
use serde::{de::Error, Deserialize, Deserializer};

use crate::cli;
use crate::io;

pub use cli::TaxonomyOptions;

#[derive(Debug)]
struct Name {
    pub tax_id: String,
    pub name: String,
    pub class: String,
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
        let class = parts
            .next()
            .ok_or_else(|| D::Error::custom("missing class"))?
            .parse()
            .map_err(D::Error::custom)?;

        Ok(Name {
            tax_id,
            name,
            class,
        })
    }
}

#[derive(Debug)]
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

    let mut nodes = vec![];
    let mut names = vec![];

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
                    Ok(n) => nodes.push(n),
                    Err(err) => eprintln!("{}", err),
                }
            }
        }
    }
    println!("processed {} nodes", nodes.len());

    if let Ok(lines) = io::read_lines(names_file) {
        for line in lines {
            if let Ok(s) = line {
                let name = serde_json::from_str::<Name>(
                    format!("\"{}\"", s.trim_end_matches("\t|"))
                        .replace("\t|\t", " , ")
                        .replace("\t\t", " , ")
                        .as_str(),
                );
                println!("{:?}", name);
                match name {
                    Ok(n) => names.push(n),
                    Err(err) => eprintln!("{}", err),
                }
            }
        }
    }
    Ok(())
}

// mod string {
//     use std::fmt::Display;
//     use std::str::FromStr;

//     use serde::{de, Deserialize, Deserializer, Serializer};

//     pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         T: Display,
//         S: Serializer,
//     {
//         serializer.collect_str(value)
//     }

//     pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
//     where
//         T: FromStr,
//         T::Err: Display,
//         D: Deserializer<'de>,
//     {
//         String::deserialize(deserializer)?
//             .parse()
//             .map_err(de::Error::custom)
//     }
// }

// #[derive(Debug, Serialize)]
// struct Node {
//     #[serde(deserialize_with = "string")]
//     pub tax_id: String,
//     #[serde(deserialize_with = "string")]
//     pub parent_id: String,
//     #[serde(deserialize_with = "string")]
//     pub rank: String,
// }

// impl<'de> Deserialize<'de> for Node {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         println!("{:?}", "here");
//         let s: &str = Deserialize::deserialize(deserializer)?;

//         println!("{:?}", "there");

//         let mut parts = s.trim().splitn(2, ",").fuse();
//         println!("{:?}", parts);
//         let tax_id: String = parts
//             .next()
//             .ok_or_else(|| D::Error::custom("missing taxId"))?
//             .into();
//         let parent_id: String = parts
//             .next()
//             .ok_or_else(|| D::Error::custom("missing parent taxId"))?
//             .into();
//         let rank: String = parts
//             .next()
//             .ok_or_else(|| D::Error::custom("missing rank"))?
//             .into();

//         Ok(Node {
//             tax_id,
//             parent_id,
//             rank,
//         })
//     }
// }
