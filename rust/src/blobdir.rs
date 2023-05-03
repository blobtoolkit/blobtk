use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use flate2::read::GzDecoder;
use glob::glob;
use serde;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;
use serde_json;
use serde_with::{serde_as, DefaultOnError};
use titlecase::titlecase;
use url::Url;

use crate::cli;
// use crate::io;

pub use cli::PlotOptions;

fn default_accession() -> String {
    "draft".to_string()
}

fn default_level() -> String {
    "scaffold".to_string()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AssemblyMeta {
    #[serde(default = "default_accession")]
    pub accession: String,
    #[serde(default = "default_level")]
    pub level: String,
    pub prefix: String,
    pub alias: Option<String>,
    pub bioproject: Option<String>,
    pub biosample: Option<String>,
    pub file: Option<PathBuf>,
    #[serde(rename = "scaffold-count")]
    pub scaffold_count: Option<usize>,
    pub span: Option<usize>,
    pub url: Option<Url>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Datatype {
    Float,
    Integer,
    Mixed,
    String,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FieldMeta {
    pub id: String,
    #[serde(rename = "type")]
    pub field_type: Option<String>,
    pub scale: Option<String>,
    pub datatype: Option<Datatype>,
    pub children: Option<Vec<FieldMeta>>,
    pub parent: Option<String>,
    pub data: Option<Vec<FieldMeta>>,
    pub count: Option<usize>,
    pub range: Option<[f64; 2]>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub clamp: Option<f64>,
    pub preload: Option<bool>,
    pub active: Option<bool>,
    #[serde(rename = "set")]
    pub odb_set: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlotMeta {
    pub x: Option<String>,
    pub y: Option<String>,
    pub z: Option<String>,
    pub cat: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaxonMeta {
    pub name: String,
    pub class: Option<String>,
    pub family: Option<String>,
    pub genus: Option<String>,
    pub kingdom: Option<String>,
    pub order: Option<String>,
    pub phylum: Option<String>,
    pub superkingdom: Option<String>,
    #[serde(deserialize_with = "deserialize_string_from_number")]
    pub taxid: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Meta {
    pub id: String,
    pub name: String,
    pub record_type: String,
    pub records: usize,
    #[serde(default = "default_revision")]
    pub revision: u8,
    #[serde(default = "default_version")]
    pub version: u8,
    pub assembly: AssemblyMeta,
    pub fields: Vec<FieldMeta>,
    pub plot: PlotMeta,
    pub taxon: TaxonMeta,
    pub field_list: Option<HashMap<String, FieldMeta>>,
    pub busco_list: Option<Vec<(String, usize, String)>>,
}

fn default_revision() -> u8 {
    0
}

fn default_version() -> u8 {
    1
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Field<T> {
    // pub meta: FieldMeta,
    pub values: Vec<T>,
    pub keys: Vec<String>,
    pub category_slot: Option<u8>,
    pub headers: Option<Vec<String>>,
}

impl<T> Field<T> {
    pub fn values(&self) -> &Vec<T> {
        &self.values
    }
}

#[derive(Debug)]
pub struct Filter {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub invert: bool,
    pub key: Option<Vec<usize>>,
}

impl Default for Filter {
    fn default() -> Filter {
        Filter {
            min: None,
            max: None,
            invert: false,
            key: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BuscoGene {
    pub id: String,
    pub status: String,
}

pub fn get_path(dir: &PathBuf, prefix: &str) -> Option<String> {
    let mut path = dir.clone();
    path.push(prefix);
    for e in glob(&format!("{}*", path.to_string_lossy())).expect("Failed to read glob pattern") {
        return Some(format!("{}", e.unwrap().to_string_lossy()));
    }
    None
}

pub fn file_reader(dir: &PathBuf, prefix: &str) -> Option<Box<dyn BufRead>> {
    let path = match get_path(dir, prefix) {
        Some(string) => string,
        None => return None,
    };
    let file = File::open(&path).expect("no such file");

    if path.ends_with(".gz") {
        return Some(Box::new(BufReader::new(GzDecoder::new(file))));
    } else {
        return Some(Box::new(BufReader::new(file)));
    };
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Keys {
    pub headers: String,
}

pub fn parse_blobdir(blobdir: &PathBuf) -> Meta {
    let reader = file_reader(blobdir, "meta.json").unwrap();
    let mut meta: Meta = serde_json::from_reader(reader).expect("unable to parse json");
    let mut fields: HashMap<String, FieldMeta> = HashMap::new();
    let mut busco_fields: Vec<(String, usize, String)> = vec![];
    fn list_fields(
        field_list: &Vec<FieldMeta>,
        fields: &mut HashMap<String, FieldMeta>,
        busco_fields: &mut Vec<(String, usize, String)>,
        busco: bool,
        parent: Option<&FieldMeta>,
    ) {
        for f in field_list {
            // let iterable_headers: HashMap<String, String> =
            //     serde_json::from_value(serde_json::to_value(&f).unwrap()).unwrap();
            let full = if parent.is_none() {
                f.clone()
            } else {
                let mut tmp = FieldMeta {
                    id: f.id.clone(),
                    children: f.children.clone(),
                    parent: f.parent.clone(),
                    data: f.data.clone(),
                    ..parent.unwrap().clone()
                };
                if f.field_type.is_some() {
                    tmp.field_type = f.field_type.clone()
                }
                if f.range.is_some() {
                    tmp.range = f.range.clone()
                }
                if f.preload.is_some() {
                    tmp.preload = f.preload.clone()
                }
                if f.active.is_some() {
                    tmp.active = f.active.clone()
                }
                if f.scale.is_some() {
                    tmp.scale = f.scale.clone()
                }
                if f.datatype.is_some() {
                    tmp.datatype = f.datatype.clone()
                }
                if f.count.is_some() {
                    tmp.count = f.count.clone()
                }
                if f.odb_set.is_some() {
                    tmp.odb_set = f.odb_set.clone()
                }
                tmp
            };

            // full = FieldMeta { id: f.id, ..parent };
            let busco_flag = if f.id == "busco".to_string() {
                true
            } else {
                busco
            };
            if f.children.is_none() {
                fields.insert(f.id.clone(), full.clone());
                if busco_flag {
                    busco_fields.push((
                        f.id.clone(),
                        f.count.unwrap_or(1),
                        f.odb_set.clone().unwrap(),
                    ));
                }
            } else {
                list_fields(
                    &f.children.clone().unwrap(),
                    fields,
                    busco_fields,
                    busco_flag,
                    Some(&full),
                )
            }
            if f.data.is_some() {
                list_fields(
                    &f.data.clone().unwrap(),
                    fields,
                    busco_fields,
                    busco_flag,
                    Some(&full),
                )
            }
        }
    }
    list_fields(&meta.fields, &mut fields, &mut busco_fields, false, None);
    meta.field_list = Some(fields);
    meta.busco_list = Some(busco_fields);
    if meta.record_type != "scaffold" {
        meta.record_type = if titlecase(&meta.assembly.level) == "Contig".to_string() {
            "contig".to_string()
        } else {
            "scaffold".to_string()
        };
    }

    meta
}

pub fn parse_field_busco(id: String, blobdir: &PathBuf) -> Option<Vec<Vec<BuscoGene>>> {
    let reader = match file_reader(blobdir, &format!("{}.json", &id)) {
        Some(reader) => reader,
        None => return None,
    };
    let field: Field<Vec<(String, usize)>> =
        serde_json::from_reader(reader).expect("unable to parse json");
    let mut values: Vec<Vec<BuscoGene>> = vec![];
    let keys = field.keys.clone();
    // let cat_slot = field.category_slot.unwrap() as usize;
    for value in field.values() {
        let mut val = vec![];
        for v in value {
            val.push(BuscoGene {
                id: v.0.clone(),
                status: keys[v.1].clone(),
            });
        }
        values.push(val);
    }
    Some(values)
}

pub fn parse_field_cat(id: String, blobdir: &PathBuf) -> Option<Vec<(String, usize)>> {
    let reader = match file_reader(blobdir, &format!("{}.json", &id)) {
        Some(reader) => reader,
        None => return None,
    };
    let field: Field<usize> = serde_json::from_reader(reader).expect("unable to parse json");
    let mut values: Vec<(String, usize)> = vec![];
    let keys = field.keys.clone();
    for value in field.values() {
        values.push((keys[*value].clone(), *value))
    }
    Some(values)
}

pub fn parse_field_float(id: String, blobdir: &PathBuf) -> Option<Vec<f64>> {
    let reader = match file_reader(blobdir, &format!("{}.json", &id)) {
        Some(reader) => reader,
        None => return None,
    };
    let field: Field<f64> = serde_json::from_reader(reader).expect("unable to parse json");
    let values = field.values().clone();
    Some(values)
}

pub fn parse_field_int(id: String, blobdir: &PathBuf) -> Option<Vec<usize>> {
    let reader = match file_reader(blobdir, &format!("{}.json", &id)) {
        Some(reader) => reader,
        None => return None,
    };
    let field: Field<usize> = serde_json::from_reader(reader).expect("unable to parse json");
    let values = field.values().clone();
    Some(values)
}

pub fn parse_field_string(id: String, blobdir: &PathBuf) -> Option<Vec<String>> {
    let reader = match file_reader(blobdir, &format!("{}.json", &id)) {
        Some(reader) => reader,
        None => return None,
    };
    let field: Field<String> = serde_json::from_reader(reader).expect("unable to parse json");
    let values = field.values().clone();
    Some(values)
}

pub fn parse_filters(filters: &Vec<String>) -> HashMap<&str, Filter> {
    let mut filter_map = HashMap::new();
    for filter in filters {
        if let Some((id, parameter)) = filter.split_once("--") {
            if !filter_map.contains_key(id) {
                filter_map.insert(
                    id,
                    Filter {
                        ..Default::default()
                    },
                );
            };
            let filter_params = filter_map.get_mut(&id).unwrap();
            if parameter == "Inv" {
                filter_params.invert = true;
                continue;
            };
            if let Some((param, value)) = parameter.split_once("=") {
                match param {
                    "Max" => filter_params.max = Some(value.parse().unwrap()),
                    "Min" => filter_params.min = Some(value.parse().unwrap()),
                    "Key" => {
                        filter_params.key = Some(
                            value
                                .split(",")
                                .map(|x| x.parse::<usize>().unwrap())
                                .collect(),
                        )
                    }
                    _ => (),
                }
            }
        };
    }

    filter_map
}

// TODO: add filters for int and cat values
pub fn filter_float_values(values: Vec<f64>, filter: Filter, indices: Vec<usize>) -> Vec<usize> {
    let initial: Vec<usize> = if indices.is_empty() {
        (0..(values.len() - 1)).collect()
    } else {
        indices.clone()
    };
    let mut output = vec![];
    for i in initial {
        let mut keep = true;
        if filter.max.is_some() {
            if values[i] > filter.max.unwrap() {
                keep = false;
            }
        }
        if filter.min.is_some() {
            if values[i] < filter.min.unwrap() {
                keep = false;
            }
        }
        if filter.invert {
            keep = !keep;
        }
        if keep {
            output.push(i);
        }
    }
    output
}

pub fn filter_int_values(values: Vec<usize>, filter: Filter, indices: Vec<usize>) -> Vec<usize> {
    let initial: Vec<usize> = if indices.is_empty() {
        (0..(values.len() - 1)).collect()
    } else {
        indices.clone()
    };
    let mut output = vec![];
    for i in initial {
        let mut keep = true;
        if filter.max.is_some() {
            if values[i] as f64 > filter.max.unwrap() {
                keep = false;
            }
        }
        if filter.min.is_some() {
            if (values[i] as f64) < filter.min.unwrap() {
                keep = false;
            }
        }
        if filter.invert {
            keep = !keep;
        }
        if keep {
            output.push(i);
        }
    }
    output
}

pub fn set_filters(filters: HashMap<&str, Filter>, meta: &Meta, blobdir: &PathBuf) -> Vec<usize> {
    let mut indices = vec![];
    let field_list = meta.field_list.clone().unwrap();
    for (id, filter) in filters {
        let field_meta_option = field_list.get(id);
        match field_meta_option {
            Some(field_meta) => {
                let field = field_meta.clone();
                match field.datatype {
                    Some(Datatype::Float) => {
                        let values = parse_field_float(field_meta.id.clone(), blobdir).unwrap();
                        indices = filter_float_values(values, filter, indices);
                    }
                    Some(Datatype::Integer) => {
                        let values = parse_field_int(field_meta.id.clone(), blobdir).unwrap();
                        indices = filter_int_values(values, filter, indices);
                    }
                    Some(_) => (),
                    None => (),
                }
            }
            None => (),
        };
    }
    if indices.is_empty() {
        indices = (0..meta.records).collect();
    }
    indices
}

pub fn apply_filter_float(values: &Vec<f64>, indices: &Vec<usize>) -> Vec<f64> {
    let mut output = vec![];
    for i in indices {
        output.push(values[i.clone()])
    }
    output
}

pub fn apply_filter_int(values: &Vec<usize>, indices: &Vec<usize>) -> Vec<usize> {
    let mut output = vec![];
    for i in indices {
        output.push(values[i.clone()])
    }
    output
}

pub fn apply_filter_busco(
    values: &Vec<Vec<BuscoGene>>,
    indices: &Vec<usize>,
) -> Vec<Vec<BuscoGene>> {
    let mut output = vec![];
    for i in indices {
        output.push(values[i.clone()].clone())
    }
    output
}

pub fn apply_filter_cat(values: &Vec<(String, usize)>, indices: &Vec<usize>) -> Vec<String> {
    let mut output = vec![];
    for i in indices {
        output.push(values[i.clone()].clone().0)
    }
    output
}

pub fn get_plot_values(
    meta: &Meta,
    blobdir: &PathBuf,
    plot_map: &HashMap<String, String>,
) -> (HashMap<String, Vec<f64>>, Vec<(String, usize)>) {
    let mut plot_values = HashMap::new();
    let mut cat_values = vec![];
    let field_list = meta.field_list.clone().unwrap();
    for (axis, id) in plot_map {
        let field_meta_option = field_list.get(id);
        match field_meta_option {
            Some(field_meta) => {
                let field = field_meta.clone();
                match field.datatype {
                    Some(Datatype::Float) => {
                        let values = parse_field_float(field_meta.id.clone(), blobdir).unwrap();
                        plot_values.insert(axis.clone(), values);
                    }
                    Some(Datatype::Integer) => {
                        let values: Vec<f64> = parse_field_int(field_meta.id.clone(), blobdir)
                            .unwrap()
                            .iter()
                            .map(|x| x.clone() as f64)
                            .collect();
                        plot_values.insert(axis.clone(), values);
                    }
                    Some(Datatype::String) => {
                        if field.data.is_some() {
                            cat_values = parse_field_cat(field_meta.id.clone(), blobdir).unwrap();
                        }
                    }
                    Some(_) => (),
                    None => (),
                }
            }
            None => (),
        };
    }
    (plot_values, cat_values)
}
