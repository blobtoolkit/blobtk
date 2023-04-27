use std::borrow::Borrow;
use std::collections::HashMap;
use std::default;
use std::str::FromStr;

use svg::node::element::{Circle, Group, Rectangle};
use svg::Document;

use crate::blobdir::FieldMeta;
use crate::utils::{max_float, scale_float, scale_floats};
use crate::{blobdir, cli, plot};

use plot::category::Category;

use super::axis::{AxisName, AxisOptions, Position, Scale};
use super::component::{chart_axis, hist_paths};
use super::plot_data::{self, Bin, HistogramData, ScatterData, ScatterPoint};
use super::style::path_filled;

#[derive(Clone, Debug)]
pub struct BlobData {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub z: Vec<f64>,
    pub cat: Vec<usize>,
    pub cat_order: Vec<Category>,
}

#[derive(Clone, Debug)]
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
            padding: [50.0, 50.0, 50.0, 50.0],
            hist_height: 200.0,
            hist_width: 200.0,
        }
    }
}

fn scale_values_new(data: &Vec<f64>, meta: &AxisOptions) -> Vec<f64> {
    let mut scaled = vec![];
    for value in data {
        scaled.push(scale_floats(
            *value,
            &meta.domain,
            &meta.range,
            &meta.scale,
            meta.clamp.clone(),
        ));
    }
    scaled
}

pub fn bin_axis(
    scatter_data: &ScatterData,
    blob_data: &BlobData,
    axis: AxisName,
    dimensions: &BlobDimensions,
    options: &cli::PlotOptions,
) -> (Vec<HistogramData>, f64) {
    let range = match axis {
        AxisName::X => scatter_data.x.range.clone(),
        AxisName::Y => scatter_data.y.range.clone(),
        AxisName::Z => scatter_data.z.range.clone(),
        _ => [0.0, 100.0],
    };
    let bin_size = (range[1] - range[0]) / options.resolution as f64;
    let mut binned = vec![vec![0.0; options.resolution]; options.cat_count];
    let z_values = &blob_data.z;
    let mut max_bin = 0.0;
    for (i, cat_index) in blob_data.cat.iter().enumerate() {
        let point = &scatter_data.points[i];
        let mut bin = match axis {
            AxisName::X => ((point.x - range[0]) / bin_size).floor() as usize,
            AxisName::Y => ((point.y - range[0]) / bin_size).floor() as usize,
            AxisName::Z => ((point.z - range[0]) / bin_size).floor() as usize,
            _ => 0,
        };
        if bin == options.resolution {
            bin -= 1;
        }
        binned[*cat_index][bin] += z_values[i];
        max_bin = max_float(max_bin, binned[*cat_index][bin]);
    }
    let width = dimensions.width / options.resolution as f64;
    let domain = [0.0, max_bin];
    let range = match axis {
        AxisName::Y => [0.0, dimensions.hist_width],
        _ => [0.0, dimensions.hist_height],
    };
    let mut histograms = vec![
        HistogramData {
            max_bin,
            ..Default::default()
        };
        blob_data.cat_order.len()
    ];
    for (i, cat) in blob_data.cat_order.iter().enumerate() {
        histograms[i] = HistogramData {
            bins: binned[i]
                .iter()
                .map(|value| Bin {
                    height: scale_floats(*value, &domain, &range, &Scale::LINEAR, None),
                    width,
                    value: *value,
                })
                .collect(),
            max_bin,
            axis: axis.clone(),
            category: Some(cat.clone()),
        }
    }
    (histograms, max_bin)
}

pub fn blob_points(
    axes: HashMap<String, String>,
    blob_data: &BlobData,
    meta: &blobdir::Meta,
    options: &cli::PlotOptions,
) -> ScatterData {
    let dimensions = BlobDimensions {
        ..Default::default()
    };
    let default_clamp = 0.1;
    let fields = meta.field_list.clone().unwrap();
    let x_meta = fields[axes["x"].as_str()].clone();
    let mut x_domain = x_meta.range.unwrap();
    let x_clamp = if x_meta.clamp.is_some() {
        x_domain[0] = x_meta.range.unwrap()[0];
        x_meta.clamp
    } else if x_meta.range.unwrap()[0] == 0.0
        && x_meta.scale.clone().unwrap() == "scaleLog".to_string()
    {
        x_domain[0] = default_clamp;
        Some(default_clamp)
    } else {
        None
    };
    let x_axis = AxisOptions {
        position: Position::BOTTOM,
        label: axes["x"].clone(),
        padding: [dimensions.padding[3], dimensions.padding[1]],
        offset: dimensions.height
            + dimensions.hist_height
            + dimensions.padding[0]
            + dimensions.padding[2],
        scale: Scale::from_str(&x_meta.scale.unwrap()).unwrap(),
        domain: x_meta.range.unwrap(),
        range: [0.0, dimensions.width],
        clamp: x_clamp,
        ..Default::default()
    };
    let x_scaled = scale_values_new(&blob_data.x, &x_axis);

    let y_meta = fields[axes["y"].as_str()].clone();
    let mut y_domain = y_meta.range.unwrap();
    let y_clamp = if y_meta.clamp.is_some() {
        y_domain[0] = y_meta.range.unwrap()[0];
        y_meta.clamp
    } else if y_meta.range.unwrap()[0] == 0.0
        && y_meta.scale.clone().unwrap() == "scaleLog".to_string()
    {
        y_domain[0] = default_clamp;
        Some(default_clamp)
    } else {
        None
    };
    let y_axis = AxisOptions {
        position: Position::LEFT,
        label: axes["y"].clone(),
        padding: [dimensions.padding[0], dimensions.padding[2]],
        scale: Scale::from_str(&y_meta.scale.unwrap()).unwrap(),
        domain: y_domain,
        range: [
            dimensions.height + dimensions.hist_height,
            dimensions.hist_height,
        ],
        clamp: y_clamp,
        ..Default::default()
    };
    let y_scaled = scale_values_new(&blob_data.y, &y_axis);

    let z_meta = fields[axes["z"].as_str()].clone();
    let z_axis = AxisOptions {
        label: axes["z"].clone(),
        scale: Scale::from_str("scaleSqrt").unwrap(),
        domain: z_meta.range.unwrap(),
        range: [2.0, dimensions.height / 15.0],
        ..Default::default()
    };
    let z_scaled = scale_values_new(&blob_data.z, &z_axis);

    let mut points = vec![];
    let cat_order = blob_data.cat_order.clone();
    let mut ordered_points = vec![vec![]; cat_order.len()];
    for (i, cat_index) in blob_data.cat.iter().enumerate() {
        let cat = cat_order[*cat_index].borrow();
        ordered_points[*cat_index].push(ScatterPoint {
            x: x_scaled[i],
            y: y_scaled[i],
            z: z_scaled[i],
            label: Some(cat.label.clone()),
            color: Some(cat.color.clone()),
        })
    }
    for cat_points in ordered_points {
        points.extend(cat_points);
    }
    ScatterData {
        points,
        x: x_axis,
        y: y_axis,
        z: z_axis,
        categories: blob_data.cat_order.clone(),
    }
}

pub fn svg(
    dimensions: &BlobDimensions,
    scatter_data: &ScatterData,
    hist_data_x: &Vec<HistogramData>,
    hist_data_y: &Vec<HistogramData>,
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
    let mut x_hist_group = Group::new().set(
        "transform",
        format!(
            "translate({}, {})",
            dimensions.padding[3],
            dimensions.padding[0] + dimensions.hist_height
        ),
    );
    for hist in hist_data_x {
        let color;
        color = hist.category.clone().unwrap().color;
        x_hist_group = x_hist_group.add(path_filled(hist.clone().to_path_data(true), Some(&color)));
    }

    // let x_hist = hist_paths(
    //     hist_data_x,
    //     [0.0, dimensions.width],
    //     [0.0, dimensions.hist_height],
    // );
    let x_axis = chart_axis(&scatter_data.x);
    let y_axis = chart_axis(&scatter_data.y);
    // let y_axis_options = AxisOptions {
    //     ..Default::default()
    // };
    // let y_axis = create_axis(&scatter_data.y, y_axis_options);

    let blob_group = Group::new()
        .set(
            "transform",
            format!(
                "translate({}, {})",
                dimensions.margin[3], dimensions.margin[0]
            ),
        )
        .add(scatter_group)
        .add(x_hist_group)
        .add(x_axis)
        .add(y_axis);
    let height = dimensions.height
        + dimensions.hist_height
        + dimensions.margin[0]
        + dimensions.margin[2]
        + dimensions.padding[0]
        + dimensions.padding[2];
    let width = dimensions.width
        + dimensions.hist_width
        + dimensions.margin[1]
        + dimensions.margin[3]
        + dimensions.padding[1]
        + dimensions.padding[3];
    let document = Document::new()
        .set("viewBox", (0, 0, height, width))
        .add(
            Rectangle::new()
                .set("fill", "#ffffff")
                .set("stroke", "none")
                .set("width", height)
                .set("height", width),
        )
        .add(blob_group);
    document
}
