use std::borrow::Borrow;
use std::collections::HashMap;

use std::str::FromStr;

use svg::node::element::{Circle, Group, Rectangle};
use svg::Document;

use crate::utils::{linear_scale, linear_scale_float, max_float, scale_floats};
use crate::{blobdir, cli, plot};

use plot::category::Category;

use super::axis::{AxisName, AxisOptions, ChartAxes, Position, Scale};
use super::blob::category_legend_full;
use super::chart::{Chart, Dimensions};
use super::component::{chart_axis, legend, LegendShape};
use super::data::{Bin, HistogramData, Line, LineData, ScatterPoint};
use super::style::{path_filled, path_open};

#[derive(Clone, Debug)]
pub struct CumulativeData {
    pub values: Vec<f64>,
    pub cat: Vec<usize>,
    pub cat_order: Vec<Category>,
}

pub fn cumulative_lines(cumulative_data: &CumulativeData, _options: &cli::PlotOptions) -> LineData {
    let dimensions = Dimensions {
        ..Default::default()
    };
    let x_domain = [0.0, cumulative_data.values.len() as f64];
    let x_range = [0.0, dimensions.width];
    let x_axis = AxisOptions {
        position: Position::BOTTOM,
        label: "cumulative count".to_string(),
        padding: [dimensions.padding[3], dimensions.padding[1]],
        offset: dimensions.height + dimensions.padding[0] + dimensions.padding[2],
        scale: Scale::LINEAR,
        domain: x_domain,
        range: x_range,
        ..Default::default()
    };
    let y_domain = [0.0, cumulative_data.values.iter().sum::<f64>()];
    let y_range = [dimensions.height, 0.0];
    let y_axis = AxisOptions {
        position: Position::LEFT,
        label: "cumulative length".to_string(),
        padding: [dimensions.padding[2], dimensions.padding[0]],
        scale: Scale::LINEAR,
        domain: y_domain,
        range: y_range,
        rotate: true,
        ..Default::default()
    };
    let mut lines = vec![];
    let cat_order = cumulative_data.cat_order.clone();
    // let mut ordered_points = vec![vec![]; cat_order.len()];
    let mut end_coords = [0.0, y_range[0]];
    for (index, cat) in cat_order.iter().enumerate() {
        let mut lengths = vec![];
        for i in cat.indices.iter() {
            lengths.push(cumulative_data.values[*i]);
        }
        lengths.sort_by(|a, b| b.partial_cmp(a).unwrap());
        let mut coords = vec![end_coords.clone()];
        let mut cumulative_span = 0.0;
        for (i, length) in lengths.iter().enumerate() {
            // add coords to line
            cumulative_span += length;
            coords.push([
                linear_scale_float((i + 1) as f64, &x_domain, &x_range),
                linear_scale_float(cumulative_span as f64, &y_domain, &y_range),
            ]);
        }
        lines.push(Line {
            coords,
            label: Some(cat.title.clone()),
            color: Some(cat.color.clone()),
            weight: 3.0,
            cat_index: index,
            ..Default::default()
        })
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
    LineData {
        lines,
        x: x_axis,
        y: y_axis,
        categories: cumulative_data.cat_order.clone(),
    }
}

pub fn plot(dimensions: Dimensions, line_data: LineData, _options: &cli::PlotOptions) -> Document {
    let height = dimensions.height
        + dimensions.margin[0]
        + dimensions.margin[2]
        + dimensions.padding[0]
        + dimensions.padding[2];

    let width = dimensions.width
        + dimensions.margin[1]
        + dimensions.margin[3]
        + dimensions.padding[1]
        + dimensions.padding[3];
    let x_opts = line_data.x.clone();
    let y_opts = line_data.y.clone();

    let cumulative = Chart {
        axes: ChartAxes {
            x: Some(x_opts.clone()),
            y: Some(y_opts.clone()),
            ..Default::default()
        },
        line_data: Some(line_data.clone()),
        dimensions: Dimensions {
            height: dimensions.height,
            width: dimensions.width,
            margin: dimensions.margin,
            padding: dimensions.padding,
        },
        ..Default::default()
    };

    let document = Document::new()
        .set("viewBox", (0, 0, width, height))
        .add(
            Rectangle::new()
                .set("fill", "#ffffff")
                .set("stroke", "none")
                .set("width", width)
                .set("height", height),
        )
        .add(cumulative.svg().set(
            "transform",
            format!(
                "translate({}, {})",
                dimensions.margin[3], dimensions.margin[0]
            ),
        ))
        .add(
            category_legend_full(line_data.categories.clone(), true).set(
                "transform",
                format!(
                    "translate({}, {})",
                    width - 185.0,
                    height
                        - dimensions.margin[2]
                        - dimensions.padding[2]
                        - line_data.categories.len() as f64 * 26.0
                ),
            ),
        );

    document
}