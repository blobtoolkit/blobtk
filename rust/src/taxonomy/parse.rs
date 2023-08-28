// use nom::bytes::complete::tag;
// use nom::sequence::delimited;

// let mut parser = tag("|");

// println!("{}", parser(line));

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::io::Write;
use std::path::PathBuf;

use anyhow;
use convert_case::{Case, Casing};
use csv::ReaderBuilder;
use nom::{
    bytes::complete::{tag, take_until},
    combinator::map,
    multi::separated_list0,
    IResult,
};
// use serde::Deserialize;

use struct_iterable::Iterable;

use crate::io;

/// A taxon name
#[derive(Clone, Debug, Default, Eq, Iterable, PartialEq)]
pub struct Name {
    pub tax_id: String,
    pub name: String,
    pub unique_name: String,
    pub class: Option<String>,
}

impl Name {
    /// Parse a node.
    pub fn parse(input: &str) -> IResult<&str, Self> {
        // This parser outputs a Vec(&str).
        let parse_name = separated_list0(tag("\t|\t"), take_until("\t|"));
        // Map the Vec(&str) into a Node.
        map(parse_name, |v: Vec<&str>| Name {
            tax_id: v[0].to_string(),
            name: v[1].to_string(),
            class: Some(v[3].to_string()),
            ..Default::default()
        })(input)
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut values = vec![];
        for (_field_name, field_value) in self.iter() {
            if let Some(string_opt) = field_value.downcast_ref::<Option<String>>() {
                if let Some(string) = string_opt.as_deref() {
                    values.push(format!("{}", string));
                } else {
                    values.push("".to_string());
                }
            } else if let Some(string_opt) = field_value.downcast_ref::<u32>() {
                values.push(format!("{:?}", string_opt));
            } else if let Some(string_opt) = field_value.downcast_ref::<String>() {
                values.push(string_opt.clone());
            }
        }
        write!(f, "{}\t|", values.join("\t|\t"))
    }
}

/// A taxonomy node
#[derive(Clone, Debug, Default, Eq, Iterable, PartialEq)]
pub struct Node {
    pub tax_id: String,
    pub parent_tax_id: String,
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
        map(parse_node, |v: Vec<&str>| Node {
            tax_id: v[0].to_string(),
            parent_tax_id: v[1].to_string(),
            rank: v[2].to_string(),
            ..Default::default()
        })(input)
    }

    pub fn tax_id(&self) -> String {
        self.tax_id.clone()
    }

    pub fn rank(&self) -> String {
        self.rank.clone()
    }

    pub fn rank_letter(&self) -> char {
        if self.rank == "subspecies" {
            return 'b';
        }
        self.rank.chars().next().unwrap()
    }

    pub fn scientific_name(&self) -> String {
        match self.scientific_name.as_ref() {
            Some(name) => name.clone(),
            None => "".to_string(),
        }
    }

    pub fn lc_tax_id(&self) -> String {
        self.tax_id.to_case(Case::Lower)
    }

    pub fn lc_scientific_name(&self) -> String {
        self.scientific_name().to_case(Case::Lower)
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ignore = vec!["names", "scientific_name"];
        let mut values = vec![];
        for (field_name, field_value) in self.iter() {
            if !ignore.contains(&field_name) {
                //     values.push(format!("{:?}", field_value.to_string()));
                // }
                if let Some(string_opt) = field_value.downcast_ref::<Option<String>>() {
                    if let Some(string) = string_opt.as_deref() {
                        values.push(format!("{:?}", string));
                    } else {
                        values.push("".to_string());
                    }
                } else if let Some(string_opt) = field_value.downcast_ref::<u32>() {
                    values.push(format!("{:?}", string_opt));
                } else if let Some(string_opt) = field_value.downcast_ref::<String>() {
                    values.push(string_opt.clone());
                }
            }
        }
        write!(f, "{}\t|", values.join("\t|\t"))
    }
}

/// A set of taxonomy nodes
#[derive(Clone, Debug, Default, Eq, Iterable, PartialEq)]
pub struct Nodes {
    pub nodes: HashMap<String, Node>,
    pub children: HashMap<String, Vec<String>>,
}

impl Nodes {
    /// Get parent Node.
    pub fn parent(&self, taxon_id: &String) -> Option<&Node> {
        let node = self.nodes.get(taxon_id).unwrap();
        self.nodes.get(&node.parent_tax_id)
    }

    /// Get lineage from root to target.
    pub fn lineage(&self, root_id: &String, taxon_id: &String) -> Vec<&Node> {
        let mut nodes = vec![];
        let mut tax_id = taxon_id;
        if tax_id == root_id {
            return nodes;
        }
        let mut prev_tax_id = tax_id.clone();
        while tax_id != root_id {
            if let Some(node) = self.parent(&tax_id) {
                tax_id = &node.tax_id;
                nodes.push(node)
            } else {
                break;
            }
            if tax_id == &prev_tax_id {
                break;
            }
            prev_tax_id = tax_id.clone();
        }
        nodes.into_iter().rev().collect()
    }

    /// Write nodes.dmp file for a root taxon.
    pub fn write_taxdump(
        &self,
        root_ids: Vec<String>,
        base_id: Option<String>,
        nodes_writer: &mut Box<dyn Write>,
        names_writer: &mut Box<dyn Write>,
    ) -> () {
        let mut ancestors = HashSet::new();
        for root_id in root_ids {
            if let Some(lineage_root_id) = base_id.clone() {
                let lineage = self.lineage(&lineage_root_id, &root_id);
                for anc_node in lineage {
                    if !ancestors.contains(&anc_node.tax_id.clone()) {
                        writeln!(nodes_writer, "{}", &anc_node).unwrap();
                        if let Some(names) = anc_node.names.as_ref() {
                            for name in names {
                                writeln!(names_writer, "{}", &name).unwrap();
                            }
                        }
                        ancestors.insert(anc_node.tax_id.clone());
                    }
                }
            }
            if let Some(root_node) = self.nodes.get(&root_id) {
                writeln!(nodes_writer, "{}", &root_node).unwrap();
                if let Some(names) = root_node.names.as_ref() {
                    for name in names {
                        writeln!(names_writer, "{}", &name).unwrap();
                    }
                }
                if let Some(children) = self.children.get(&root_id) {
                    for child in children {
                        self.write_taxdump(vec![child.clone()], None, nodes_writer, names_writer)
                    }
                }
            }
        }
    }
}

pub fn parse_taxdump(taxdump: PathBuf) -> Result<Nodes, anyhow::Error> {
    let mut nodes = HashMap::new();
    let mut children = HashMap::new();

    let mut nodes_file = taxdump.clone();
    nodes_file.push("nodes.dmp");

    // Parse nodes.dmp file
    if let Ok(lines) = io::read_lines(nodes_file) {
        for line in lines {
            if let Ok(s) = line {
                let node = Node::parse(&s).unwrap().1;
                let parent = node.parent_tax_id.clone();
                let child = node.tax_id.clone();
                if parent != child {
                    match children.entry(parent) {
                        Entry::Vacant(e) => {
                            e.insert(vec![child]);
                        }
                        Entry::Occupied(mut e) => {
                            e.get_mut().push(child);
                        }
                    }
                }

                nodes.insert(node.tax_id.clone(), node);
            }
        }
    }

    let mut names_file = taxdump.clone();
    names_file.push("names.dmp");

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

    Ok(Nodes { nodes, children })
}

pub fn write_taxdump(
    nodes: &Nodes,
    root_taxon_ids: Option<Vec<String>>,
    base_taxon_id: Option<String>,
    taxdump: PathBuf,
) {
    let mut root_ids = vec![];
    match root_taxon_ids {
        Some(ids) => {
            for id in ids {
                root_ids.push(id)
            }
        }
        None => root_ids.push("1".to_string()),
    };
    let mut nodes_writer = io::get_writer(&Some(io::append_to_path(&taxdump, "/nodes.dmp")));
    let mut names_writer = io::get_writer(&Some(io::append_to_path(&taxdump, "/names.dmp")));

    nodes.write_taxdump(
        root_ids,
        base_taxon_id,
        &mut nodes_writer,
        &mut names_writer,
    );
}

pub fn parse_gbif(gbif_backbone: PathBuf) -> Result<Nodes, anyhow::Error> {
    let mut nodes = HashMap::new();
    let mut children = HashMap::new();

    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_path(gbif_backbone)?;

    // Status can be:
    // ACCEPTED
    // DOUBTFUL
    // HETEROTYPIC_SYNONYM
    // HOMOTYPIC_SYNONYM
    // MISAPPLIED
    // PROPARTE_SYNONYM
    // SYNONYM
    let mut ignore = HashSet::new();
    ignore.insert("DOUBTFUL");
    ignore.insert("MISAPPLIED");
    for result in rdr.records() {
        let record = result?;
        let status = record.get(4).unwrap();
        if ignore.contains(status) {
            continue;
        }

        let tax_id = record.get(0).unwrap().to_string();
        let name_class = match status {
            "ACCEPTED" => "scientific name".to_string(),
            _ => "synonym".to_string(),
        };
        let taxon_name = record.get(19).unwrap().to_string();
        let mut parent_tax_id = record.get(1).unwrap().to_string();
        if parent_tax_id == "\\N" {
            parent_tax_id = tax_id.clone()
        }
        let name = Name {
            tax_id: tax_id.clone(),
            name: taxon_name.clone(),
            class: Some(name_class.clone()),
            ..Default::default()
        };
        match nodes.entry(tax_id.clone()) {
            Entry::Vacant(e) => {
                let node = Node {
                    tax_id,
                    parent_tax_id,
                    rank: record.get(5).unwrap().to_case(Case::Lower),
                    scientific_name: if name_class == "scientific name" {
                        Some(taxon_name)
                    } else {
                        None
                    },
                    names: Some(vec![name]),
                    ..Default::default()
                };
                let parent = node.parent_tax_id.clone();
                let child = node.tax_id.clone();
                if parent != child {
                    match children.entry(parent) {
                        Entry::Vacant(e) => {
                            e.insert(vec![child]);
                        }
                        Entry::Occupied(mut e) => {
                            e.get_mut().push(child);
                        }
                    }
                }

                e.insert(node);
            }
            Entry::Occupied(mut e) => {
                if name_class == "scientific name" {
                    e.get_mut().scientific_name = Some(taxon_name);
                }
                if let Some(names) = e.get_mut().names.as_mut() {
                    names.push(name);
                }
            }
        }

        // println!("{:?}", record.get(0));
        // let node = Node {
        //     tax_id,
        //     parent_tax_id: record.get(1).unwrap().to_string(),
        //     rank: record.get(5).unwrap().to_case(Case::Lower),
        //     scientific_name: Some(record.get(19).unwrap().to_string()),
        //     ..Default::default()
        // };
    }
    Ok(Nodes { nodes, children })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_name() {
        assert_eq!(
            Name::parse("1	|	all	|		|	synonym	|").unwrap(),
            (
                "\t|",
                Name {
                    tax_id: String::from("1"),
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
                    tax_id: String::from("1"),
                    parent_tax_id: String::from("1"),
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
                    tax_id: String::from("2"),
                    parent_tax_id: String::from("131567"),
                    rank: String::from("superkingdom"),
                    ..Default::default()
                }
            )
        );
    }
}
