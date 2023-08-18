// use nom::bytes::complete::tag;
// use nom::sequence::delimited;

// let mut parser = tag("|");

// println!("{}", parser(line));

use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow;
use nom::{
    bytes::complete::{tag, take_until},
    combinator::map,
    multi::separated_list0,
    IResult,
};

use crate::io;

/// A taxon name
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Name {
    pub tax_id: u32,
    pub name: String,
    pub class: Option<String>,
}

impl Name {
    /// Parse a node.
    pub fn parse(input: &str) -> IResult<&str, Self> {
        // This parser outputs a Vec(&str).
        let parse_name = separated_list0(tag("\t|\t"), take_until("\t|"));
        // Map the Vec(&str) into a Node.
        map(parse_name, |v| Name {
            tax_id: u32::from_str(v[0]).unwrap(),
            name: v[1].to_string(),
            class: Some(v[3].to_string()),
            ..Default::default()
        })(input)
    }
}

/// A taxonomy node
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Node {
    pub tax_id: u32,
    pub parent_tax_id: u32,
    pub rank: String,
    pub names: Option<Vec<Name>>,
    pub scientific_name: Option<String>,
}

impl Node {
    /// Parse a node.
    pub fn parse(input: &str) -> IResult<&str, Self> {
        // This parser outputs a Vec(&str).
        let parse_node = separated_list0(tag("\t|\t"), take_until("\t|"));
        // Map the Vec(&str) into a Node.
        map(parse_node, |v| Node {
            tax_id: u32::from_str(v[0]).unwrap(),
            parent_tax_id: u32::from_str(v[1]).unwrap(),
            rank: v[2].to_string(),
            ..Default::default()
        })(input)
    }
}

pub fn parse_taxdump(taxdump: &Option<PathBuf>) -> Result<HashMap<u32, Node>, anyhow::Error> {
    let mut nodes = HashMap::new();

    let nodes_file = match taxdump.clone() {
        Some(mut d) => {
            d.push("nodes.dmp");
            d
        }
        None => return Ok(nodes),
    };

    // Parse nodes.dmp file
    if let Ok(lines) = io::read_lines(nodes_file) {
        for line in lines {
            if let Ok(s) = line {
                let node = Node::parse(&s).unwrap().1;
                nodes.insert(node.tax_id, node);
            }
        }
    }

    let names_file = match taxdump.clone() {
        Some(mut d) => {
            d.push("names.dmp");
            d
        }
        None => return Ok(nodes),
    };

    // Parse names.dmp file and add to nodes
    if let Ok(lines) = io::read_lines(names_file) {
        for line in lines {
            if let Ok(s) = line {
                let name = Name::parse(&s).unwrap().1;
                let mut node = nodes.get_mut(&name.tax_id).unwrap();
                if let Some(class) = name.clone().class {
                    if class == "scientific name" {
                        node.scientific_name = Some(name.clone().name)
                    }
                }
                let mut names = node.names.as_mut();
                if let Some(names) = names.as_mut() {
                    names.push(name);
                } else {
                    node.names = Some(vec![name]);
                }
            }
        }
    }

    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ayla
    // Edward

    #[test]
    fn test_parse_name() {
        assert_eq!(
            Name::parse("1	|	all	|		|	synonym	|").unwrap(),
            (
                "\t|",
                Name {
                    tax_id: 1,
                    name: String::from("all"),
                    class: Some(String::from("synonym")),
                    ..Default::default()
                }
            )
        );
    }

    #[test]
    fn test_parse_node() {
        assert_eq!(
            Node::parse("1	|	1	|	no rank	|").unwrap(),
            (
                "\t|",
                Node {
                    tax_id: 1,
                    parent_tax_id: 1,
                    rank: String::from("no rank"),
                    ..Default::default()
                }
            )
        );
        assert_eq!(
            Node::parse("2	|	131567	|	superkingdom	|		|	0	|	0	|	11	|	0	|	0	|	0	|	0	|	0	|		|").unwrap(),
            (
                "\t|",
                Node {
                    tax_id: 2,
                    parent_tax_id: 131567,
                    rank: String::from("superkingdom"),
                    ..Default::default()
                }
            )
        );
    }
}
