// use nom::bytes::complete::tag;
// use nom::sequence::delimited;

// let mut parser = tag("|");

// println!("{}", parser(line));

use std::borrow::BorrowMut;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow;
use convert_case::{Case, Casing};
use cpc::{eval, units::Unit};
use csv::{ReaderBuilder, StringRecord};
use nom::{
    bytes::complete::{tag, take_until},
    combinator::map,
    multi::separated_list0,
    IResult,
};
use regex::Regex;
use serde;
use serde::{Deserialize, Deserializer, Serialize};

use struct_iterable::Iterable;

use crate::error;
use crate::io;

use super::lookup::build_lookup;

/// A taxon name
#[derive(Clone, Debug, Default, Eq, Iterable, Ord, PartialEq, PartialOrd)]
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
#[derive(Clone, Debug, Default, Eq, Iterable, Ord, PartialEq, PartialOrd)]
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

    pub fn names_by_class(&self, classes_vec: Option<&Vec<String>>, lc: bool) -> Vec<String> {
        let mut filtered_names = vec![];
        if let Some(names) = self.names.clone() {
            for name in names {
                if let Some(classes) = classes_vec {
                    if let Some(class) = name.class {
                        if classes.contains(&class) {
                            if lc {
                                filtered_names.push(name.name.to_case(Case::Lower));
                            } else {
                                filtered_names.push(name.name.clone());
                            }
                        }
                    }
                } else if lc {
                    filtered_names.push(name.name.to_case(Case::Lower));
                } else {
                    filtered_names.push(name.name.clone());
                }
            }
        }
        filtered_names
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

    pub fn nodes_by_rank(&self, rank: &str) -> Vec<Node> {
        let mut nodes = vec![];
        for node in self.nodes.iter() {
            if node.1.rank == rank {
                nodes.push(node.1.clone());
            }
        }
        nodes
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
                let node = nodes.get_mut(&name.tax_id).unwrap();
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

    nodes.insert(
        "root".to_string(),
        Node {
            tax_id: "root".to_string(),
            parent_tax_id: "root".to_string(),
            rank: "root".to_string(),
            scientific_name: None,
            names: None,
            ..Default::default()
        },
    );

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
    ignore.insert("HETEROTYPIC_SYNONYM");
    ignore.insert("HOMOTYPIC_SYNONYM");
    ignore.insert("PROPARTE_SYNONYM");
    ignore.insert("SYNONYM");
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
            parent_tax_id = "root".to_string()
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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum GHubsFileFormat {
    #[serde(rename = "csv")]
    CSV,
    #[default]
    #[serde(rename = "tsv")]
    TSV,
}

impl FromStr for GHubsFileFormat {
    type Err = ();
    fn from_str(input: &str) -> Result<GHubsFileFormat, Self::Err> {
        match input {
            "csv" => Ok(GHubsFileFormat::CSV),
            "csv.gz" => Ok(GHubsFileFormat::CSV),
            "tsv" => Ok(GHubsFileFormat::TSV),
            "tsv.gz" => Ok(GHubsFileFormat::TSV),
            _ => Err(()),
        }
    }
}

// Value may be String or Vec of Strings
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrVec {
    Single(String),
    Multiple(Vec<String>),
}

// Value may be u32 or Vec of u32
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UsizeOrVec {
    Single(usize),
    Multiple(Vec<usize>),
}

// Value may be PathBuf or Vec of PathBuf
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PathBufOrVec {
    Single(PathBuf),
    Multiple(Vec<PathBuf>),
}

// Field types
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum FieldType {
    #[serde(rename = "byte")]
    Byte,
    #[serde(rename = "date")]
    Date,
    #[serde(rename = "double")]
    Double,
    #[serde(rename = "float")]
    Float,
    #[serde(rename = "geo_point")]
    GeoPoint,
    #[serde(rename = "half_float")]
    HalfFloat,
    #[default]
    #[serde(rename = "keyword")]
    Keyword,
    #[serde(rename = "integer")]
    Integer,
    #[serde(rename = "long")]
    Long,
    #[serde(rename = "short")]
    Short,
    #[serde(rename = "1dp")]
    OneDP,
    #[serde(rename = "2dp")]
    TwoDP,
    #[serde(rename = "3dp")]
    ThreeDP,
    #[serde(rename = "4dp")]
    FourDP,
}

/// GenomeHubs file configuration options
#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct GHubsFileConfig {
    /// File format
    pub format: GHubsFileFormat,
    /// Flag to indicate whether file has a header row
    pub header: bool,
    /// Filename or path relative to the configuration file
    pub name: PathBuf,
    /// Additional configuration files that must be loaded
    pub needs: Option<PathBufOrVec>,
}

/// GenomeHubs field constraint configuration options
#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct ConstraintConfig {
    // List of valid values
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<String>>,
    // Value length
    pub len: Option<usize>,
    // Maximum value
    pub max: Option<f64>,
    // Minimum value
    pub min: Option<f64>,
}

// Field types
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub enum FieldScale {
    #[default]
    #[serde(rename = "linear")]
    Linear,
    #[serde(rename = "log2")]
    Log2,
    #[serde(rename = "log10")]
    Log10,
    #[serde(rename = "double")]
    SQRT,
}

/// GenomeHubs value bins configuration options
#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct BinsConfig {
    // List of valid values
    pub count: u32,
    // Geographic resolution (hexagonal)
    pub h3res: Option<u8>,
    // Maximum value
    pub max: f64,
    // Minimum value
    pub min: f64,
    // Value length
    pub scale: FieldScale,
}

/// GenomeHubs field display configuration options
#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct DisplayConfig {
    // Display group
    pub group: Option<String>,
    // Display level
    pub level: Option<u8>,
    // Displa name
    pub name: Option<String>,
}

/// GenomeHubs field status values
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub enum FieldStatus {
    // Temporary
    #[default]
    #[serde(rename = "temporary")]
    Temporary,
}

/// GenomeHubs field configuration options
#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct GHubsFieldConfig {
    // Default settings for value bins
    pub bins: Option<BinsConfig>,
    // Constraint on field values
    pub constraint: Option<ConstraintConfig>,
    // Default value
    pub default: Option<String>,
    // Field description
    pub description: Option<String>,
    // Display options
    pub display: Option<DisplayConfig>,
    // Function to apply to value
    pub function: Option<String>,
    // Column header
    pub header: Option<StringOrVec>,
    // Column index
    pub index: Option<UsizeOrVec>,
    // String to join columns
    pub join: Option<String>,
    // Attribute key
    pub key: Option<String>,
    // Attribute name
    pub name: Option<String>,
    // Value separator
    pub separator: Option<StringOrVec>,
    // Attribute status
    pub status: Option<FieldStatus>,
    // Attribute summary functions
    pub summary: Option<StringOrVec>,
    // Attribute name synonyms
    #[serde(alias = "synonym")]
    pub synonyms: Option<StringOrVec>,
    // List of values to translate
    pub translate: Option<HashMap<String, StringOrVec>>,
    // Field type
    #[serde(rename = "type", default = "default_field_type")]
    pub field_type: FieldType,
    // Attribute value units
    #[serde(alias = "unit")]
    pub units: Option<String>,
}

fn default_field_type() -> FieldType {
    FieldType::Keyword
}

impl GHubsFieldConfig {
    fn merge(self, other: GHubsFieldConfig) -> Self {
        Self {
            bins: self.bins.or(other.bins),
            constraint: self.constraint.or(other.constraint),
            default: self.default.or(other.default),
            description: self.description.or(other.description),
            display: self.display.or(other.display),
            function: self.function.or(other.function),
            header: self.header.or(other.header),
            index: self.index.or(other.index),
            join: self.join.or(other.join),
            key: self.key.or(other.key),
            name: self.name.or(other.name),
            separator: self.separator.or(other.separator),
            status: self.status.or(other.status),
            summary: self.summary.or(other.summary),
            synonyms: self.synonyms.or(other.synonyms),
            translate: self.translate.or(other.translate),
            field_type: self.field_type,
            units: self.units.or(other.units),
        }
    }
}

/// Merges 2 GenomeHubs configuration files
fn merge_attributes(
    self_attributes: Option<HashMap<String, GHubsFieldConfig>>,
    other_attributes: Option<HashMap<String, GHubsFieldConfig>>,
    merged_attributes: &mut HashMap<String, GHubsFieldConfig>,
) {
    if let Some(attributes) = self_attributes {
        if other_attributes.is_some() {
            let new_attributes = other_attributes.unwrap();
            for (field, other_config) in new_attributes.clone() {
                if let Some(config) = attributes.get(&field) {
                    merged_attributes.insert(field.clone(), config.clone().merge(other_config));
                } else {
                    merged_attributes.insert(field.clone(), other_config.clone());
                }
            }
            for (field, config) in attributes {
                if let Some(_) = new_attributes.get(&field) {
                    continue;
                } else {
                    merged_attributes.insert(field.clone(), config.clone());
                }
            }
        } else {
            for (field, config) in attributes {
                merged_attributes.insert(field.clone(), config.clone());
            }
        }
    } else if let Some(attributes) = other_attributes {
        for (field, config) in attributes {
            merged_attributes.insert(field.clone(), config.clone());
        }
    }
}

/// GenomeHubs configuration options
#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct GHubsConfig {
    /// File configuration options
    pub file: Option<GHubsFileConfig>,
    /// Attribute fields
    pub attributes: Option<HashMap<String, GHubsFieldConfig>>,
    /// Taxonomy fields
    pub taxonomy: Option<HashMap<String, GHubsFieldConfig>>,
}

impl GHubsConfig {
    pub fn get(&self, key: &str) -> Option<&HashMap<String, GHubsFieldConfig>> {
        match key {
            "attributes" => self.attributes.as_ref(),
            "taxonomy" => self.taxonomy.as_ref(),
            _ => None,
        }
    }
    pub fn get_mut(&mut self, key: &str) -> Option<&mut HashMap<String, GHubsFieldConfig>> {
        match key {
            "attributes" => self.attributes.as_mut(),
            "taxonomy" => self.taxonomy.as_mut(),
            _ => None,
        }
    }
    fn merge(self, other: GHubsConfig) -> Self {
        let mut merged_attributes = HashMap::new();
        let self_attributes = self.attributes;
        let other_attributes = other.attributes;
        merge_attributes(self_attributes, other_attributes, &mut merged_attributes);
        let mut merged_taxonomy = HashMap::new();
        let self_taxonomy = self.taxonomy;
        let other_taxonomy = other.taxonomy;
        merge_attributes(self_taxonomy, other_taxonomy, &mut merged_taxonomy);
        Self {
            file: self.file.or(other.file),
            attributes: Some(merged_attributes),
            taxonomy: Some(merged_taxonomy),
        }
    }
}

// Parse a GenomeHubs configuration file
fn parse_genomehubs_config(config_file: &PathBuf) -> Result<GHubsConfig, error::Error> {
    let reader = match io::file_reader(config_file.clone()) {
        Some(r) => r,
        None => {
            return Err(error::Error::FileNotFound(format!(
                "{}",
                &config_file.to_str().unwrap()
            )))
        }
    };
    let mut ghubs_config: GHubsConfig = match serde_yaml::from_reader(reader) {
        Ok(options) => options,
        Err(err) => {
            return Err(error::Error::SerdeError(format!(
                "{} {}",
                &config_file.to_str().unwrap(),
                err.to_string()
            )))
        }
    };
    if let Some(file_config) = &ghubs_config.file {
        if let Some(needs) = &file_config.needs {
            let mut base_path = config_file.clone();
            base_path.pop();
            let needs_files = match needs {
                PathBufOrVec::Single(file) => {
                    base_path.push(file);
                    vec![base_path]
                }
                PathBufOrVec::Multiple(files) => {
                    let mut needs_paths = vec![];
                    for file in files.iter() {
                        let mut needs_path = base_path.clone();
                        needs_path.push(file);
                        needs_paths.push(needs_path);
                    }
                    needs_paths
                }
            };
            for needs_file in needs_files.iter() {
                let extra_config = match parse_genomehubs_config(&needs_file) {
                    Ok(extra_config) => extra_config,
                    Err(err) => return Err(err),
                };
                // TODO: combine_configs(extra_config, ghubs_config);
                ghubs_config = extra_config.merge(ghubs_config.clone());
            }
        }
    }
    Ok(ghubs_config)
}

fn key_index(headers: &StringRecord, key: &str) -> Result<usize, error::Error> {
    match headers.iter().position(|column| column == key) {
        Some(index) => Ok(index),
        None => Err(error::Error::IndexError(format!(
            "Column '{}' does not exist.",
            key
        ))),
    }
}

fn update_config(key: &str, ghubs_config: &mut GHubsConfig, headers: &StringRecord) {
    for (_, field) in ghubs_config.borrow_mut().get_mut(key).unwrap().iter_mut() {
        if field.header.is_some() {
            // if let Some(header) = &field.header {
            // let field_idx = &mut field.index;
            field.index = match &field.header.as_ref().unwrap().clone() {
                StringOrVec::Single(item) => Some(UsizeOrVec::Single(
                    key_index(headers, item.as_str()).unwrap(),
                )),
                StringOrVec::Multiple(list) => Some(UsizeOrVec::Multiple(
                    list.iter()
                        .map(|item| key_index(headers, item.as_str()).unwrap())
                        .collect::<Vec<usize>>(),
                )),
            };
            // field.index = field_index;
        };
    }
}

fn check_bounds<T: Into<f64> + Copy>(value: &T, constraint: &ConstraintConfig) -> bool {
    let val: f64 = Into::<f64>::into(value.to_owned());
    if let Some(min) = constraint.min {
        if val < min {
            eprintln!("Value {} is less than minimum {}", val, min);
            return false;
        }
    }
    if let Some(max) = constraint.max {
        if val > max {
            eprintln!("Value {} is greater than maximum {}", val, max);
            return false;
        }
    }
    true
}

// fn apply_constraint(value: &mut GHubsConfig, constraint: &ConstraintConfig) {}

fn validate_double(value: &String, constraint: &ConstraintConfig) -> Result<bool, error::Error> {
    let v = value
        .parse::<f64>()
        .map_err(|_| error::Error::ParseError(format!("Invalid double value: {}", value)))?;
    Ok(check_bounds(&v, constraint))
}

fn apply_validation(value: String, field: &GHubsFieldConfig) -> Result<bool, error::Error> {
    let constraint = match field.constraint.to_owned() {
        Some(c) => c,
        None => ConstraintConfig {
            ..Default::default()
        },
    };
    // let mut valid = false;
    // dbg!(&field);
    let ref field_type = field.field_type;
    let valid = match field_type {
        FieldType::Byte => {
            let dot_pos = value.find(".").unwrap_or(value.len());
            let v = value[..dot_pos]
                .parse::<i8>()
                .map_err(|_| error::Error::ParseError(format!("Invalid byte value: {}", value)))?;
            check_bounds(&v, &constraint)
        }
        FieldType::Date => true,
        FieldType::Double => validate_double(&value, &constraint)?,

        FieldType::Float => {
            let v = value
                .parse::<f32>()
                .map_err(|_| error::Error::ParseError(format!("Invalid float value: {}", value)))?;
            check_bounds(&v, &constraint)
        }
        FieldType::GeoPoint => true,
        FieldType::HalfFloat => {
            let v = value.parse::<f32>().map_err(|_| {
                error::Error::ParseError(format!("Invalid half_float value: {}", value))
            })?;
            check_bounds(&v, &constraint)
        }
        FieldType::Keyword => true,
        FieldType::Integer => {
            let dot_pos = value.find(".").unwrap_or(value.len());
            let v = value[..dot_pos].parse::<i32>().map_err(|_| {
                error::Error::ParseError(format!("Invalid integer value: {}", value))
            })?;
            check_bounds(&v, &constraint)
        }
        FieldType::Long => {
            let dot_pos = value.find(".").unwrap_or(value.len());
            value[..dot_pos]
                .parse::<i64>()
                .map_err(|_| error::Error::ParseError(format!("Invalid long value: {}", value)))?;
            validate_double(&value, &constraint)?
        }
        FieldType::Short => {
            let dot_pos = value.find(".").unwrap_or(value.len());
            let v = value[..dot_pos]
                .parse::<i16>()
                .map_err(|_| error::Error::ParseError(format!("Invalid short value: {}", value)))?;
            check_bounds(&v, &constraint)
        }
        FieldType::OneDP => true,
        FieldType::TwoDP => true,
        FieldType::ThreeDP => true,
        FieldType::FourDP => true,
    };
    Ok(valid)
}

fn apply_function(value: String, field: &GHubsFieldConfig) -> String {
    if value == "" {
        return "None".to_string();
    }
    let mut val = value;
    if let Some(ref function) = field.function {
        let equation = function.replace("{}", val.as_str());
        let value = eval(equation.as_str(), false, Unit::NoUnit, false).unwrap();
        val = format!("{}", value);
    }
    match apply_validation(val.clone(), &field) {
        Ok(is_valid) => {
            if is_valid {
                val
            } else {
                "None".to_string()
            }
        }
        Err(_) => "None".to_string(),
    }
}

fn translate_value(field: &GHubsFieldConfig, value: &String) -> Vec<String> {
    let mut values = vec![];
    // dbg!(&field.header, &value);
    if let Some(ref translate) = field.translate {
        let translated = translate
            .get(value)
            .cloned()
            .unwrap_or(StringOrVec::Single(value.to_owned()));
        match translated {
            StringOrVec::Single(val) => values.push(val),
            StringOrVec::Multiple(vals) => values.extend(vals),
        };
    } else {
        values.push(value.to_owned());
    }
    values
}

fn process_value(value: String, field: &GHubsFieldConfig) -> Result<Vec<String>, error::Error> {
    let values = translate_value(field, &value);
    let mut ret_values = vec![];
    for value in values {
        if let Some(separator) = &field.separator {
            let re = match separator {
                StringOrVec::Single(sep) => Regex::new(sep).unwrap(),
                StringOrVec::Multiple(separators) => Regex::new(
                    separators
                        // .iter()
                        // .map(|sep| record.get(idx.to_owned()).unwrap_or(""))
                        // .collect::<Vec<&str>>()
                        .join(&"|")
                        .as_str(),
                )
                .unwrap(),
            };
            for val in re.split(value.as_str()) {
                ret_values.push(apply_function(val.to_string(), &field));
            }
        } else {
            ret_values.push(apply_function(value, &field));
        }
    }
    Ok(ret_values)
}

fn validate_values(
    key: &str,
    ghubs_config: &mut GHubsConfig,
    record: &StringRecord,
) -> HashMap<String, String> {
    let mut validated = HashMap::new();
    for (field_name, field) in ghubs_config.borrow_mut().get_mut(key).unwrap().iter_mut() {
        if let Some(index) = &field.index {
            let string_value = match index {
                UsizeOrVec::Single(idx) => record.get(idx.to_owned()).unwrap().to_string(),
                UsizeOrVec::Multiple(indices) => indices
                    .iter()
                    .map(|idx| record.get(idx.to_owned()).unwrap_or(""))
                    .collect::<Vec<&str>>()
                    .join(&field.join.as_ref().unwrap_or(&"".to_string())),
            };
            let values = process_value(string_value, field).unwrap().join(";");
            validated.insert(field_name.clone(), values);
        }
    }
    validated
}

// Parse taxa from a GenomeHubs data file
fn nodes_from_file(
    config_file: &PathBuf,
    ghubs_config: &mut GHubsConfig,
    _lookup_table: &HashMap<String, Vec<String>>,
) -> Result<(), error::Error> {
    let file_config = ghubs_config.file.as_ref().unwrap();
    let delimiter = match file_config.format {
        GHubsFileFormat::CSV => b',',
        GHubsFileFormat::TSV => b'\t',
    };
    let mut path = config_file.clone();
    path.pop();
    path.push(file_config.name.clone());

    let mut rdr = ReaderBuilder::new()
        .has_headers(file_config.header)
        .delimiter(delimiter)
        .from_path(path)?;
    let headers = rdr.headers()?;
    let keys = vec!["attributes", "taxonomy"];
    for key in keys.iter() {
        if ghubs_config.get(key).is_some() {
            update_config(key, ghubs_config, headers);
        }
    }
    // dbg!(&ghubs_config);

    for result in rdr.records() {
        let record = result?;
        let mut processed = HashMap::new();
        for key in keys.iter() {
            if ghubs_config.get(key).is_some() {
                let value = validate_values(key, ghubs_config, &record);
                processed.insert(key, value);
            }
        }
        // let status = record.get(4).unwrap();
        dbg!(processed);
    }
    Ok(())
}

pub fn parse_file(
    config_file: PathBuf,
    lookup_table: &HashMap<String, Vec<String>>,
) -> Result<(), error::Error> {
    // let mut children = HashMap::new();

    let mut ghubs_config = match parse_genomehubs_config(&config_file) {
        Ok(ghubs_config) => ghubs_config,
        Err(err) => return Err(err),
    };
    let nodes = nodes_from_file(&config_file, &mut ghubs_config, &lookup_table);
    dbg!(&nodes);

    // let mut rdr = ReaderBuilder::new()
    //     .has_headers(false)
    //     .delimiter(b'\t')
    //     .from_path(gbif_backbone)?;

    Ok(())
}

/// Deserializer for lineage
fn lineage_deserialize<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let str_sequence = String::deserialize(deserializer)?;
    Ok(str_sequence
        .split(';')
        .map(|item| item.trim().to_owned())
        .collect())
}

/// ENA taxonomy record from taxonomy API
#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct EnaTaxon {
    // Unique taxon ID
    #[serde(rename = "taxId")]
    pub tax_id: String,
    // Scientific name
    #[serde(rename = "scientificName")]
    pub scientific_name: String,
    // Taxonomic rank
    pub rank: String,
    // Lineage
    #[serde(deserialize_with = "lineage_deserialize")]
    pub lineage: Vec<String>,
}

pub fn parse_ena_jsonl(
    jsonl: PathBuf,
    existing: Option<&mut Nodes>,
) -> Result<Nodes, error::Error> {
    let mut nodes = HashMap::new();
    let mut children = HashMap::new();
    let name_classes = vec!["scientific name".to_string()];
    if let Some(existing_nodes) = existing {
        let table = build_lookup(existing_nodes, &name_classes, false);

        let lines = match io::read_lines(&jsonl) {
            Ok(r) => r,
            Err(_) => {
                return Err(error::Error::FileNotFound(format!(
                    "{:?}",
                    &jsonl.as_os_str()
                )))
            }
        };

        for line in lines {
            if let Ok(json) = line {
                let taxon: EnaTaxon = serde_json::from_str(&json)?;
                let scientific_name = taxon.scientific_name;
                for names in taxon
                    .lineage
                    .into_iter()
                    .rev()
                    .collect::<Vec<String>>()
                    .windows(2)
                {
                    let key = format!(
                        "{}:{}",
                        names[0].to_case(Case::Lower),
                        names[1].to_case(Case::Lower)
                    );
                    if let Some(parent_tax_ids) = table.get(&key) {
                        if parent_tax_ids.len() == 1 {
                            let node = Node {
                                tax_id: taxon.tax_id.clone(),
                                parent_tax_id: parent_tax_ids[0].clone(),
                                rank: taxon.rank,
                                scientific_name: Some(scientific_name.clone()),
                                names: Some(vec![Name {
                                    tax_id: taxon.tax_id.clone(),
                                    name: scientific_name,
                                    class: Some("scientific name".to_string()),
                                    ..Default::default()
                                }]),
                            };
                            existing_nodes.nodes.insert(taxon.tax_id.clone(), node);
                            match existing_nodes.children.entry(parent_tax_ids[0].clone()) {
                                Entry::Vacant(e) => {
                                    e.insert(vec![taxon.tax_id]);
                                }
                                Entry::Occupied(mut e) => {
                                    e.get_mut().push(taxon.tax_id);
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
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
