use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use serde;
use serde::{Deserialize, Deserializer, Serialize};
use serde_aux::prelude::*;
use serde_json;
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
    pub busco_list: Option<Vec<String>>,
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
pub struct BuscoGene {
    pub id: String,
    pub status: String,
}

fn update_fields(field_list: &Vec<FieldMeta>) {}

pub fn parse_blobdir(options: &cli::PlotOptions) -> Meta {
    let mut blob_meta = options.blobdir.clone();
    blob_meta.push("meta.json");
    let file = File::open(blob_meta).expect("no such file");
    let reader = BufReader::new(file);

    let mut meta: Meta = serde_json::from_reader(reader).expect("unable to parse json");
    // println!("dataset {} has {} records", meta.id, meta.records);
    let mut fields: Vec<String> = vec![];
    let mut busco_fields: Vec<String> = vec![];
    fn list_fields(
        field_list: &Vec<FieldMeta>,
        fields: &mut Vec<String>,
        busco_fields: &mut Vec<String>,
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
                    busco_fields.push(f.id.clone());
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

    meta
}

fn field_reader(id: &String, options: &cli::PlotOptions) -> Option<BufReader<File>> {
    let mut field_data = options.blobdir.clone();
    field_data.push(format!("{}.json", &id));
    let file = match File::open(field_data) {
        Ok(file) => file,
        Err(err) => {
            println!("No such file. {}.json could not be found", &id);
            return None;
        }
    };
    let reader = Some(BufReader::new(file));
    reader
}

pub fn parse_field_busco(id: String, options: &cli::PlotOptions) -> Option<Vec<Vec<BuscoGene>>> {
    let reader = match field_reader(&id, &options) {
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
    let reader = match field_reader(&id, &options) {
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
    let reader = match field_reader(&id, &options) {
        Some(reader) => reader,
        None => return None,
    };
    let field: Field<f64> = serde_json::from_reader(reader).expect("unable to parse json");
    let values = field.values().clone();
    Some(values)
}

pub fn parse_field_int(id: String, options: &cli::PlotOptions) -> Option<Vec<usize>> {
    let reader = match field_reader(&id, &options) {
        Some(reader) => reader,
        None => return None,
    };
    let field: Field<usize> = serde_json::from_reader(reader).expect("unable to parse json");
    let values = field.values().clone();
    Some(values)
}

pub fn parse_field_string(id: String, options: &cli::PlotOptions) -> Option<Vec<String>> {
    let reader = match field_reader(&id, &options) {
        Some(reader) => reader,
        None => return None,
    };
    let field: Field<String> = serde_json::from_reader(reader).expect("unable to parse json");
    let values = field.values().clone();
    Some(values)
}
