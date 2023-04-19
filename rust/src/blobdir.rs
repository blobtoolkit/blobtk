use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use flate2::read::GzDecoder;
use glob::glob;
use serde;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;
use serde_json;
use titlecase::titlecase;
use url::Url;

use crate::cli;
// use crate::io;

pub use cli::PlotOptions;

fn default_level() -> String {
    "scaffold".to_string()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AssemblyMeta {
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
pub struct FieldMeta {
    pub id: String,
    #[serde(rename = "type")]
    pub field_type: Option<String>,
    pub scale: Option<String>,
    pub datatype: Option<String>,
    pub children: Option<Vec<FieldMeta>>,
    pub parent: Option<String>,
    pub count: Option<usize>,
    #[serde(rename = "set")]
    pub odb_set: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
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
    pub revision: u8,
    pub version: u8,
    pub assembly: AssemblyMeta,
    pub fields: Vec<FieldMeta>,
    pub plot: PlotMeta,
    pub taxon: TaxonMeta,
    pub field_list: Option<Vec<String>>,
    pub busco_list: Option<Vec<(String, usize, String)>>,
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

pub fn parse_blobdir(options: &cli::PlotOptions) -> Meta {
    let reader = file_reader(&options.blobdir, "meta.json").unwrap();
    let mut meta: Meta = serde_json::from_reader(reader).expect("unable to parse json");
    let mut fields: Vec<String> = vec![];
    let mut busco_fields: Vec<(String, usize, String)> = vec![];
    fn list_fields(
        field_list: &Vec<FieldMeta>,
        fields: &mut Vec<String>,
        busco_fields: &mut Vec<(String, usize, String)>,
        busco: bool,
    ) {
        for f in field_list {
            let busco_flag = if f.id == "busco".to_string() {
                true
            } else {
                busco
            };
            if f.children.is_none() {
                fields.push(f.id.clone());
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
                )
            }
        }
    }
    list_fields(&meta.fields, &mut fields, &mut busco_fields, false);
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

pub fn parse_field_busco(id: String, options: &cli::PlotOptions) -> Option<Vec<Vec<BuscoGene>>> {
    let reader = match file_reader(&options.blobdir, &format!("{}.json", &id)) {
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

pub fn parse_field_cat(id: String, options: &cli::PlotOptions) -> Option<Vec<String>> {
    let reader = match file_reader(&options.blobdir, &format!("{}.json", &id)) {
        Some(reader) => reader,
        None => return None,
    };
    let field: Field<usize> = serde_json::from_reader(reader).expect("unable to parse json");
    let mut values: Vec<String> = vec![];
    let keys = field.keys.clone();
    for value in field.values() {
        values.push(keys[*value].clone())
    }
    Some(values)
}

pub fn parse_field_float(id: String, options: &cli::PlotOptions) -> Option<Vec<f64>> {
    let reader = match file_reader(&options.blobdir, &format!("{}.json", &id)) {
        Some(reader) => reader,
        None => return None,
    };
    let field: Field<f64> = serde_json::from_reader(reader).expect("unable to parse json");
    let values = field.values().clone();
    Some(values)
}

pub fn parse_field_int(id: String, options: &cli::PlotOptions) -> Option<Vec<usize>> {
    let reader = match file_reader(&options.blobdir, &format!("{}.json", &id)) {
        Some(reader) => reader,
        None => return None,
    };
    let field: Field<usize> = serde_json::from_reader(reader).expect("unable to parse json");
    let values = field.values().clone();
    Some(values)
}

pub fn parse_field_string(id: String, options: &cli::PlotOptions) -> Option<Vec<String>> {
    let reader = match file_reader(&options.blobdir, &format!("{}.json", &id)) {
        Some(reader) => reader,
        None => return None,
    };
    let field: Field<String> = serde_json::from_reader(reader).expect("unable to parse json");
    let values = field.values().clone();
    Some(values)
}
