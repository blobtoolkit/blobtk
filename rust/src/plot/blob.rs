use std::borrow::Borrow;
use std::collections::HashMap;

use svg::node::element::{Circle, Group, Rectangle};
use svg::Document;

use crate::blobdir::FieldMeta;
use crate::plot::scatter::ScatterPoint;
use crate::utils::scale_float;
use crate::{blobdir, cli, plot};

use plot::category::Category;

use super::component::{x_axis, y_axis};
use super::scatter::{ScatterAxis, ScatterData};

pub struct BlobData {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub z: Vec<f64>,
    pub cat: Vec<usize>,
    pub cat_order: Vec<Category>,
}

pub struct BlobDimensions {
    pub height: f64,
    pub width: f64,
    pub margin: [f64; 4],
    pub padding: [f64; 4],
    pub hist_height: f64,
    pub hist_width: f64,
}

impl Default for BlobDimensions {
    fn default() -> BlobDimensions {
        BlobDimensions {
            height: 900.0,
            width: 900.0,
            margin: [10.0, 10.0, 100.0, 100.0],
            padding: [25.0, 25.0, 25.0, 25.0],
            hist_height: 200.0,
            hist_width: 200.0,
        }
    }
}

fn scale_values(
    data: Vec<f64>,
    meta: FieldMeta,
    range: [f64; 2],
    scale: Option<String>,
) -> Vec<f64> {
    let domain = meta.range.unwrap();
    let scale = match scale {
        Some(s) => s,
        None => meta.scale.unwrap(),
    };
    let clamp = meta.clamp;
    let mut scaled = vec![];
    for value in data {
        scaled.push(scale_float(value, &domain, &range, &scale, clamp));
    }
    scaled
}

pub fn blob_points(
    axes: HashMap<String, String>,
    blob_data: BlobData,
    meta: &blobdir::Meta,
    options: &cli::PlotOptions,
) -> ScatterData {
    let dimensions = BlobDimensions {
        ..Default::default()
    };
    let fields = meta.field_list.clone().unwrap();
    let x_scaled = scale_values(
        blob_data.x,
        fields[axes["x"].as_str()].clone(),
        [0.0, dimensions.width],
        None,
    );
    let x_meta = fields[axes["x"].as_str()].clone();
    let x_axis = ScatterAxis {
        label: axes["x"].clone(),
        scale: x_meta.scale.unwrap(),
        domain: x_meta.range.unwrap(),
        range: [0.0, dimensions.width],
        clamp: x_meta.clamp,
    };
    let y_scaled = scale_values(
        blob_data.y,
        fields[axes["y"].as_str()].clone(),
        [dimensions.height, 0.0],
        None,
    );
    let y_meta = fields[axes["y"].as_str()].clone();
    let y_axis = ScatterAxis {
        label: axes["y"].clone(),
        scale: y_meta.scale.unwrap(),
        domain: y_meta.range.unwrap(),
        range: [dimensions.height, 0.0],
        clamp: y_meta.clamp,
    };
    let z_scaled = scale_values(
        blob_data.z,
        fields[axes["z"].as_str()].clone(),
        [2.0, dimensions.height / 15.0],
        Some("scaleSqrt".to_string()),
    );
    let z_meta = fields[axes["z"].as_str()].clone();
    let z_axis = ScatterAxis {
        label: axes["z"].clone(),
        scale: "scaleSqrt".to_string(),
        domain: z_meta.range.unwrap(),
        range: [2.0, dimensions.height / 15.0],
        clamp: None,
    };

    let mut points = vec![];
    let cat_order = blob_data.cat_order.clone();
    for (i, cat_index) in blob_data.cat.iter().enumerate() {
        let cat = cat_order[*cat_index].borrow();
        points.push(ScatterPoint {
            x: x_scaled[i],
            y: y_scaled[i],
            z: z_scaled[i],
            label: Some(cat.label.clone()),
            color: Some(cat.color.clone()),
        })
    }
    ScatterData {
        points,
        x: x_axis,
        y: y_axis,
        z: z_axis,
        categories: blob_data.cat_order,
    }
}

pub fn svg(
    dimensions: &BlobDimensions,
    scatter_data: &ScatterData,
    options: &cli::PlotOptions,
) -> Document {
    let mut scatter_group = Group::new().set(
        "transform",
        format!(
            "translate({}, {})",
            dimensions.padding[3], dimensions.padding[0]
        ),
    );
    for point in scatter_data.points.iter() {
        scatter_group = scatter_group.add(
            Circle::new()
                .set("cx", point.x)
                .set("cy", point.y)
                .set("r", point.z)
                .set("fill", point.color.clone().unwrap())
                .set("stroke", "#999999")
                .set("opacity", 1),
        );
    }
    let x_axis = x_axis(&scatter_data.x);
    let y_axis = y_axis(&scatter_data.y);

    let blob_group = Group::new()
        .set(
            "transform",
            format!(
                "translate({}, {})",
                dimensions.margin[3], dimensions.margin[0]
            ),
        )
        .add(scatter_group)
        .add(y_axis);
    let document = Document::new()
        .set("viewBox", (0, 0, 1000, 1000))
        .add(
            Rectangle::new()
                .set("fill", "#ffffff")
                .set("stroke", "none")
                .set("width", 1000)
                .set("height", 1000),
        )
        .add(blob_group);
    document
}
