use std::borrow::Borrow;
use std::collections::HashMap;

use std::str::FromStr;

use svg::node::element::{Circle, Group, Rectangle};
use svg::Document;

use crate::utils::{max_float, scale_floats};
use crate::{blobdir, cli, plot};

use plot::category::Category;

use super::axis::{AxisName, AxisOptions, ChartAxes, Position, Scale};
use super::chart::{Chart, Dimensions};
use super::component::{chart_axis, legend, LegendShape};
use super::data::{Bin, HistogramData, LineData, ScatterPoint};
use super::style::{path_filled, path_open};

#[derive(Clone, Debug)]
pub struct CumulativeData {
    pub z: Vec<f64>,
    pub cat: Vec<usize>,
    pub cat_order: Vec<Category>,
}

pub fn cumulative_lines(
    axes: HashMap<String, String>,
    cumulative_data: &CumulativeData,
    meta: &blobdir::Meta,
    _options: &cli::PlotOptions,
) -> LineData {
    let dimensions = Dimensions {
        ..Default::default()
    };
    // let default_clamp = 0.1;
    // let fields = meta.field_list.clone().unwrap();
    // let x_meta = fields[axes["x"].as_str()].clone();
    // let mut x_domain = x_meta.range.unwrap();
    // let x_clamp = if x_meta.clamp.is_some() {
    //     x_domain[0] = x_meta.range.unwrap()[0];
    //     x_meta.clamp
    // } else if x_meta.range.unwrap()[0] == 0.0
    //     && x_meta.scale.clone().unwrap() == "scaleLog".to_string()
    // {
    //     x_domain[0] = default_clamp;
    //     Some(default_clamp)
    // } else {
    //     None
    // };
    // let x_axis = AxisOptions {
    //     position: Position::BOTTOM,
    //     label: axes["x"].clone(),
    //     padding: [dimensions.padding[3], dimensions.padding[1]],
    //     offset: dimensions.height + dimensions.padding[0] + dimensions.padding[2],
    //     scale: Scale::from_str(&x_meta.scale.unwrap()).unwrap(),
    //     domain: x_meta.range.unwrap(),
    //     range: [0.0, dimensions.width],
    //     clamp: x_clamp,
    //     ..Default::default()
    // };
    // let x_scaled = scale_values(&blob_data.x, &x_axis);

    // let y_meta = fields[axes["y"].as_str()].clone();
    // let mut y_domain = y_meta.range.unwrap();
    // let y_clamp = if y_meta.clamp.is_some() {
    //     y_domain[0] = y_meta.range.unwrap()[0];
    //     y_meta.clamp
    // } else if y_meta.range.unwrap()[0] == 0.0
    //     && y_meta.scale.clone().unwrap() == "scaleLog".to_string()
    // {
    //     y_domain[0] = default_clamp;
    //     Some(default_clamp)
    // } else {
    //     None
    // };
    // let y_axis = AxisOptions {
    //     position: Position::LEFT,
    //     label: axes["y"].clone(),
    //     padding: [dimensions.padding[2], dimensions.padding[0]],
    //     scale: Scale::from_str(&y_meta.scale.unwrap()).unwrap(),
    //     domain: y_domain,
    //     range: [dimensions.height, 0.0],
    //     clamp: y_clamp,
    //     rotate: true,
    //     ..Default::default()
    // };
    // let y_scaled = scale_values(&blob_data.y, &y_axis);

    // let z_meta = fields[axes["z"].as_str()].clone();
    // let z_axis = AxisOptions {
    //     label: axes["z"].clone(),
    //     scale: Scale::from_str("scaleSqrt").unwrap(),
    //     domain: z_meta.range.unwrap(),
    //     range: [2.0, dimensions.height / 15.0],
    //     ..Default::default()
    // };
    // let z_scaled = scale_values(&blob_data.z, &z_axis);

    let mut lines = vec![];
    let cat_order = cumulative_data.cat_order.clone();
    // let mut ordered_points = vec![vec![]; cat_order.len()];
    for (i, cat_index) in cumulative_data.cat.iter().enumerate() {
        let cat = cat_order[*cat_index].borrow();
        //     ordered_points[*cat_index].push(ScatterPoint {
        //         x: x_scaled[i],
        //         y: y_scaled[i],
        //         z: z_scaled[i],
        //         label: Some(cat.label.clone()),
        //         color: Some(cat.color.clone()),
        //         cat_index: *cat_index,
        //         data_index: i,
        //     })
    }
    // for cat_points in ordered_points {
    //     points.extend(cat_points);
    // }
    let x_axis = AxisOptions {
        ..Default::default()
    };
    let y_axis = AxisOptions {
        ..Default::default()
    };
    LineData {
        lines,
        x: x_axis,
        y: y_axis,
        categories: cumulative_data.cat_order.clone(),
    }
}
