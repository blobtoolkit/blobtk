use std::borrow::Borrow;
use std::collections::HashMap;

use std::str::FromStr;

use svg::node::element::{Group, Rectangle};
use svg::Document;

use crate::utils::{max_float, min_float, scale_floats};
use crate::{blobdir, cli, plot};

use plot::category::Category;

use super::axis::{AxisName, AxisOptions, ChartAxes, Position, Scale};
use super::chart::{Chart, Dimensions};
use super::component::{legend_group, LegendEntry, LegendShape};
use super::data::{Bin, HistogramData, Reducer, ScatterData, ScatterPoint};
use super::ShowLegend;

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
        let dimensions = Dimensions {
            ..Default::default()
        };
        BlobDimensions {
            height: dimensions.height,
            width: dimensions.width,
            margin: dimensions.margin,
            padding: dimensions.padding,
            hist_height: 250.0,
            hist_width: 250.0,
        }
    }
}

fn scale_values(data: &Vec<f64>, meta: &AxisOptions) -> Vec<f64> {
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
    options: &cli::PlotOptions,
) -> (Vec<Vec<f64>>, f64) {
    let range = match axis {
        AxisName::X => scatter_data.x.range.clone(),
        AxisName::Y => scatter_data.y.range.clone(),
        AxisName::Z => scatter_data.z.range.clone(),
        _ => [0.0, 100.0],
    };
    let bin_size = (range[1] - range[0]) / options.resolution as f64;
    let mut binned = vec![vec![0.0; options.resolution]; options.cat_count];
    let mut counts = vec![vec![0.0; options.resolution]; options.cat_count];
    let mut max_bin = 0.0;
    for point in scatter_data.points.iter() {
        let cat_index = point.cat_index;
        let mut bin = match axis {
            AxisName::X => ((point.x - range[0]) / bin_size).floor() as usize,
            AxisName::Y => ((point.y - range[0]) / bin_size).floor() as usize,
            AxisName::Z => ((point.z - range[0]) / bin_size).floor() as usize,
            _ => 0,
        };
        if bin == options.resolution {
            bin -= 1;
        }
        match options.reducer_function {
            Reducer::Sum => binned[cat_index][bin] += blob_data.z[point.data_index],
            Reducer::Max => {
                binned[cat_index][bin] =
                    max_float(binned[cat_index][bin], blob_data.z[point.data_index])
            }
            Reducer::Min => {
                binned[cat_index][bin] = if binned[cat_index][bin] == 0.0 {
                    blob_data.z[point.data_index]
                } else {
                    min_float(binned[cat_index][bin], blob_data.z[point.data_index])
                }
            }
            Reducer::Count => binned[cat_index][bin] += 1.0,
            Reducer::Mean => {
                binned[cat_index][bin] += blob_data.z[point.data_index];
                counts[cat_index][bin] += 1.0
            }
        };
        max_bin = max_float(max_bin, binned[cat_index][bin]);
    }
    match options.reducer_function {
        Reducer::Mean => {
            max_bin = 0.0;
            for (cat_index, bins) in binned.clone().iter().enumerate() {
                for (bin, _) in bins.iter().enumerate() {
                    if counts[cat_index][bin] > 0.0 {
                        binned[cat_index][bin] /= counts[cat_index][bin];
                        max_bin = max_float(max_bin, binned[cat_index][bin]);
                    }
                }
            }
        }
        Reducer::Min => {
            max_bin = 0.0;
            for (cat_index, bins) in binned.clone().iter().enumerate() {
                for (bin, _) in bins.iter().enumerate() {
                    max_bin = max_float(max_bin, binned[cat_index][bin]);
                }
            }
        }
        _ => (),
    }
    (binned, max_bin)
}

pub fn axis_hist(
    binned: Vec<Vec<f64>>,
    blob_data: &BlobData,
    dimensions: &BlobDimensions,
    max_bin: f64,
    axis: AxisName,
    options: &cli::PlotOptions,
) -> Vec<HistogramData> {
    let domain = [0.0, max_bin];
    let (width, range) = match axis {
        AxisName::X => (dimensions.width, [0.0, dimensions.hist_height]),
        _ => (dimensions.height, [0.0, dimensions.hist_width]),
    };
    let cat_order = blob_data.cat_order.clone();
    let bin_width = width / options.resolution as f64;
    let mut histograms = vec![
        HistogramData {
            max_bin,
            width,
            ..Default::default()
        };
        cat_order.len() - 1
    ];
    for (j, cat) in cat_order.iter().enumerate() {
        if j == 0 {
            continue;
        }
        let i = j - 1;
        histograms[i] = HistogramData {
            bins: binned[i]
                .iter()
                .map(|value| Bin {
                    height: scale_floats(*value, &domain, &range, &Scale::LINEAR, None),
                    width: bin_width,
                    value: *value,
                })
                .collect(),
            max_bin: scale_floats(max_bin, &domain, &range, &Scale::LINEAR, None),
            axis: axis.clone(),
            category: Some(cat.clone()),
            ..histograms[i]
        }
    }
    histograms
}

pub fn bin_axes(
    scatter_data: &ScatterData,
    blob_data: &BlobData,
    dimensions: &BlobDimensions,
    options: &cli::PlotOptions,
) -> (Vec<HistogramData>, Vec<HistogramData>, f64) {
    let (x_binned, x_max) = bin_axis(scatter_data, blob_data, AxisName::X, options);
    let (y_binned, y_max) = bin_axis(scatter_data, blob_data, AxisName::Y, options);
    let mut max_bin = max_float(x_max, y_max);
    if options.hist_height.is_some() {
        max_bin = max_float(max_bin, options.hist_height.unwrap() as f64)
    }
    let x_histograms = axis_hist(
        x_binned,
        blob_data,
        dimensions,
        max_bin,
        AxisName::X,
        options,
    );
    let y_histograms = axis_hist(
        y_binned,
        blob_data,
        dimensions,
        max_bin,
        AxisName::Y,
        options,
    );
    (x_histograms, y_histograms, max_bin)
}

pub fn blob_points(
    axes: HashMap<String, String>,
    blob_data: &BlobData,
    dimensions: &BlobDimensions,
    meta: &blobdir::Meta,
    options: &cli::PlotOptions,
) -> ScatterData {
    let default_clamp = 0.1;
    let fields = meta.field_list.clone().unwrap();
    let x_meta = fields[axes["x"].as_str()].clone();
    let mut x_domain = x_meta.range.unwrap();
    if options.x_limit.is_some() {
        if let Some((min_value, max_value)) = options.x_limit.clone().unwrap().split_once(",") {
            if !min_value.is_empty() {
                x_domain[0] = min_value.parse::<f64>().unwrap();
            }
            if !max_value.is_empty() {
                x_domain[1] = max_value.parse::<f64>().unwrap();
            }
        }
    }
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
    if x_domain[0] == x_domain[1] {
        if x_domain[0] == 0.0 {
            x_domain[1] += 0.1;
        } else {
            x_domain[0] /= 0.1;
            x_domain[1] *= 0.1;
        }
    }
    let x_axis = AxisOptions {
        position: Position::BOTTOM,
        height: dimensions.height + dimensions.padding[0] + dimensions.padding[2],
        label: axes["x"].clone(),
        padding: [dimensions.padding[3], dimensions.padding[1]],
        offset: dimensions.height + dimensions.padding[0] + dimensions.padding[2],
        scale: Scale::from_str(&x_meta.scale.unwrap()).unwrap(),
        domain: x_domain,
        range: [0.0, dimensions.width],
        clamp: x_clamp,
        ..Default::default()
    };
    let x_scaled = scale_values(&blob_data.x, &x_axis);

    let y_meta = fields[axes["y"].as_str()].clone();
    let mut y_domain = y_meta.range.unwrap();
    if options.y_limit.is_some() {
        if let Some((min_value, max_value)) = options.y_limit.clone().unwrap().split_once(",") {
            if !min_value.is_empty() {
                y_domain[0] = min_value.parse::<f64>().unwrap();
            }
            if !max_value.is_empty() {
                y_domain[1] = max_value.parse::<f64>().unwrap();
            }
        }
    }

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

    if y_domain[0] == y_domain[1] {
        if y_domain[0] == 0.0 {
            y_domain[1] += 0.1;
        } else {
            y_domain[0] /= 2.0;
            y_domain[1] *= 2.0;
        }
    }
    let y_axis = AxisOptions {
        position: Position::LEFT,
        height: dimensions.width + dimensions.padding[1] + dimensions.padding[3],
        label: axes["y"].clone(),
        padding: [dimensions.padding[2], dimensions.padding[0]],
        scale: Scale::from_str(&y_meta.scale.unwrap()).unwrap(),
        domain: y_domain,
        range: [dimensions.height, 0.0],
        clamp: y_clamp,
        rotate: true,
        ..Default::default()
    };
    let y_scaled = scale_values(&blob_data.y, &y_axis);

    let z_meta = fields[axes["z"].as_str()].clone();
    let mut z_domain = z_meta.range.unwrap().clone();
    if z_domain[0] == z_domain[1] {
        if z_domain[0] == 0.0 {
            z_domain[1] += 0.1;
        } else {
            z_domain[0] /= 2.0;
            z_domain[1] *= 2.0;
        }
    }
    let z_axis = AxisOptions {
        label: axes["z"].clone(),
        scale: options.scale_function.clone(),
        domain: z_domain,
        range: [2.0, 2.0 + dimensions.height / 15.0 * options.scale_factor],
        ..Default::default()
    };
    let z_scaled = scale_values(&blob_data.z, &z_axis);

    let mut points = vec![];
    let cat_order = blob_data.cat_order.clone();
    let mut ordered_points = vec![vec![]; cat_order.len() - 1];
    for (i, cat_index) in blob_data.cat.iter().enumerate() {
        let cat = cat_order[*cat_index].borrow();
        ordered_points[*cat_index - 1].push(ScatterPoint {
            x: x_scaled[i],
            y: y_scaled[i],
            z: z_scaled[i],
            label: Some(cat.title.clone()),
            color: Some(cat.color.clone()),
            cat_index: *cat_index - 1,
            data_index: i,
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

pub fn category_legend_full(categories: Vec<Category>, show_legend: ShowLegend) -> Group {
    let mut entries = vec![];
    let title = "".to_string();
    match show_legend {
        ShowLegend::Full => entries.push(LegendEntry {
            subtitle: Some("[count; span; n50]".to_string()),
            shape: LegendShape::None,
            ..Default::default()
        }),
        _ => (),
    };
    for (i, cat) in categories.iter().enumerate() {
        if i == 0 {
            match show_legend {
                ShowLegend::Full => (),
                _ => continue,
            };
        }
        let subtitle = match show_legend {
            ShowLegend::Compact => None,
            ShowLegend::Default | ShowLegend::Full => Some(cat.clone().subtitle()),
            ShowLegend::None => return legend_group(title, entries, None, 1),
        };
        entries.push(LegendEntry {
            title: format!("{}", cat.title),
            color: cat.color.clone(),
            subtitle,
            ..Default::default()
        });
    }
    legend_group(title, entries, None, 1)
}

pub fn plot(
    blob_dimensions: BlobDimensions,
    scatter_data: ScatterData,
    hist_data_x: Vec<HistogramData>,
    hist_data_y: Vec<HistogramData>,
    x_max: f64,
    y_max: f64,
    options: &cli::PlotOptions,
) -> Document {
    let height = blob_dimensions.height
        + blob_dimensions.hist_height
        + blob_dimensions.margin[0]
        + blob_dimensions.margin[2]
        + blob_dimensions.padding[0]
        + blob_dimensions.padding[2];

    let width = blob_dimensions.width
        + blob_dimensions.hist_width
        + blob_dimensions.margin[1]
        + blob_dimensions.margin[3]
        + blob_dimensions.padding[1]
        + blob_dimensions.padding[3];
    let x_opts = scatter_data.x.clone();
    let y_opts = scatter_data.y.clone();

    let scatter = Chart {
        axes: ChartAxes {
            x: Some(x_opts.clone()),
            y: Some(y_opts.clone()),
            ..Default::default()
        },
        scatter_data: Some(scatter_data.clone()),
        dimensions: Dimensions {
            height: blob_dimensions.height,
            width: blob_dimensions.width,
            margin: blob_dimensions.margin,
            padding: blob_dimensions.padding,
        },
        ..Default::default()
    };

    let x_hist = Chart {
        axes: ChartAxes {
            x: Some(AxisOptions {
                label: "".to_string(),
                offset: blob_dimensions.hist_height,
                height: blob_dimensions.hist_height,
                tick_labels: false,
                ..x_opts.clone()
            }),
            y: Some(AxisOptions {
                position: Position::LEFT,
                label: "sum length".to_string(),
                label_offset: 80.0,
                height: blob_dimensions.width
                    + blob_dimensions.padding[1]
                    + blob_dimensions.padding[3],
                font_size: 25.0,
                scale: Scale::LINEAR,
                domain: [0.0, x_max],
                range: [blob_dimensions.hist_height, 0.0],
                rotate: true,
                tick_count: 5,
                ..Default::default()
            }),
            x2: Some(AxisOptions {
                offset: 0.0,
                position: Position::TOP,
                major_ticks: None,
                minor_ticks: None,
                ..x_opts.clone()
            }),
            y2: Some(AxisOptions {
                position: Position::RIGHT,
                offset: blob_dimensions.width
                    + blob_dimensions.padding[1]
                    + blob_dimensions.padding[3],
                scale: Scale::LINEAR,
                domain: [0.0, x_max],
                range: [blob_dimensions.hist_height, 0.0],
                major_ticks: None,
                minor_ticks: None,
                ..Default::default()
            }),
            ..Default::default()
        },
        histogram_data: Some(hist_data_x),
        dimensions: Dimensions {
            height: blob_dimensions.hist_height,
            width: blob_dimensions.width,
            margin: [0.0, 0.0, 0.0, 0.0],
            padding: [
                0.0,
                blob_dimensions.padding[1],
                0.0,
                blob_dimensions.padding[3],
            ],
        },
        ..Default::default()
    };

    let y_hist = Chart {
        axes: ChartAxes {
            x: Some(AxisOptions {
                offset: 0.0,
                height: blob_dimensions.hist_height,
                label: "".to_string(),
                tick_labels: false,
                ..y_opts.clone()
            }),
            y: Some(AxisOptions {
                position: Position::BOTTOM,
                height: blob_dimensions.height
                    + blob_dimensions.padding[0]
                    + blob_dimensions.padding[2],
                offset: blob_dimensions.height
                    + blob_dimensions.padding[0]
                    + blob_dimensions.padding[2],
                label: "sum length".to_string(),
                label_offset: 80.0,
                font_size: 25.0,
                scale: Scale::LINEAR,
                domain: [0.0, y_max],
                range: [0.0, blob_dimensions.hist_width],
                tick_count: 5,
                rotate: true,
                ..Default::default()
            }),
            x2: Some(AxisOptions {
                offset: blob_dimensions.hist_width,
                position: Position::RIGHT,
                major_ticks: None,
                minor_ticks: None,
                label: "".to_string(),
                ..y_opts.clone()
            }),
            y2: Some(AxisOptions {
                position: Position::TOP,
                offset: 0.0,
                scale: Scale::LINEAR,
                domain: [0.0, y_max],
                range: [0.0, blob_dimensions.hist_width],
                major_ticks: None,
                minor_ticks: None,
                label: "".to_string(),
                ..Default::default()
            }),

            ..Default::default()
        },
        histogram_data: Some(hist_data_y),
        dimensions: Dimensions {
            height: blob_dimensions.hist_width,
            width: blob_dimensions.height,
            margin: [0.0, 0.0, 0.0, 0.0],
            padding: [
                blob_dimensions.padding[0],
                0.0,
                blob_dimensions.padding[2],
                0.0,
            ],
        },
        ..Default::default()
    };

    let legend_x = match options.show_legend {
        ShowLegend::Compact => width - blob_dimensions.hist_width,
        _ => width - 185.0,
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
        .add(scatter.svg().set(
            "transform",
            format!(
                "translate({}, {})",
                blob_dimensions.margin[3],
                blob_dimensions.hist_height + blob_dimensions.margin[0]
            ),
        ))
        .add(x_hist.svg().set(
            "transform",
            format!(
                "translate({}, {})",
                blob_dimensions.margin[3], blob_dimensions.margin[0]
            ),
        ))
        .add(y_hist.svg().set(
            "transform",
            format!(
                "translate({}, {})",
                blob_dimensions.margin[3]
                    + blob_dimensions.width
                    + blob_dimensions.padding[1]
                    + blob_dimensions.padding[3],
                blob_dimensions.hist_height + blob_dimensions.margin[0]
            ),
        ))
        .add(
            category_legend_full(scatter_data.categories, options.show_legend.clone())
                .set("transform", format!("translate({}, {})", legend_x, 10.0)),
        );

    document
}

pub fn legend(
    blob_dimensions: BlobDimensions,
    scatter_data: ScatterData,
    options: &cli::PlotOptions,
) -> Document {
    let height = scatter_data.categories.len() * 26;

    let mut width =
        blob_dimensions.hist_width + blob_dimensions.margin[3] + blob_dimensions.padding[3];

    width = match options.show_legend {
        ShowLegend::Compact => width,
        _ => width + 220.0,
    };

    let offset_x = match options.show_legend {
        ShowLegend::Compact => 0.0,
        _ => width - 180.0,
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
        .add(
            category_legend_full(scatter_data.categories, options.show_legend.clone())
                .set("transform", format!("translate({}, {})", offset_x, 10.0)),
        );

    document
}

// pub fn svg(
//     dimensions: &BlobDimensions,
//     scatter_data: &ScatterData,
//     hist_data_x: &Vec<HistogramData>,
//     hist_data_y: &Vec<HistogramData>,
//     _options: &cli::PlotOptions,
// ) -> Document {
//     let mut scatter_group = Group::new().set(
//         "transform",
//         format!(
//             "translate({}, {})",
//             dimensions.padding[3], dimensions.padding[0]
//         ),
//     );
//     for point in scatter_data.points.iter() {
//         scatter_group = scatter_group.add(
//             Circle::new()
//                 .set("cx", point.x)
//                 .set("cy", point.y)
//                 .set("r", point.z)
//                 .set("fill", point.color.clone().unwrap())
//                 .set("stroke", "#999999")
//                 .set("opacity", 1),
//         );
//     }
//     let mut x_hist_group = Group::new().set(
//         "transform",
//         format!(
//             "translate({}, {})",
//             dimensions.padding[3], dimensions.hist_height
//         ),
//     );
//     for hist in hist_data_x {
//         let color;
//         color = hist.category.clone().unwrap().color;
//         x_hist_group = x_hist_group.add(path_open(
//             hist.clone().to_path_data(Position::BOTTOM, false),
//             Some(&color),
//             None,
//         ));
//     }

//     let mut y_hist_group = Group::new().set(
//         "transform",
//         format!(
//             "translate({}, {})",
//             dimensions.width + dimensions.padding[3] + dimensions.margin[3],
//             dimensions.height + dimensions.padding[0] + dimensions.hist_width
//         ),
//     );
//     for hist in hist_data_y {
//         let color;
//         color = hist.category.clone().unwrap().color;
//         y_hist_group = y_hist_group.add(path_filled(
//             hist.clone().to_path_data(Position::LEFT, true),
//             Some(&color),
//         ));
//     }

//     let x_axis = chart_axis(&scatter_data.x);
//     let y_axis = chart_axis(&scatter_data.y);

//     let blob_group = Group::new()
//         .set(
//             "transform",
//             format!(
//                 "translate({}, {})",
//                 dimensions.margin[3], dimensions.margin[0]
//             ),
//         )
//         .add(scatter_group)
//         .add(x_hist_group)
//         .add(y_hist_group)
//         .add(x_axis)
//         .add(y_axis);
//     let height = dimensions.height
//         + dimensions.hist_height
//         + dimensions.margin[0]
//         + dimensions.margin[2]
//         + dimensions.padding[0]
//         + dimensions.padding[2];
//     let width = dimensions.width
//         + dimensions.hist_width
//         + dimensions.margin[1]
//         + dimensions.margin[3]
//         + dimensions.padding[1]
//         + dimensions.padding[3];
//     let document = Document::new()
//         .set("viewBox", (0, 0, height, width))
//         .add(
//             Rectangle::new()
//                 .set("fill", "#ffffff")
//                 .set("stroke", "none")
//                 .set("width", height)
//                 .set("height", width),
//         )
//         .add(blob_group);
//     document
// }
