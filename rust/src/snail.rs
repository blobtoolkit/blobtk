use std::cmp::Ordering;
use std::f64::consts::PI;

use cli::PlotOptions;
use coord_transforms::d2::polar2cartesian;
use coord_transforms::prelude::*;
use num_integer::div_rem;
use serde;
use serde::{Deserialize, Serialize};
use svg::node::element::path::Data;
use svg::node::element::{Circle, Group, Line, Path, Text};
use svg::node::Text as nodeText;
use svg::Document;

use crate::blobdir;
use crate::cli;

mod compact_float {
    //! rounds a float to 3 decimal places, when serialized into a str, such as for JSON
    //! offers space savings when such such precision is not needed.
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(float: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{:.3}", float);
        let parsed = s.parse::<f64>().unwrap();
        serializer.serialize_f64(parsed)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        f64::deserialize(deserializer)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SummaryStats {
    #[serde(with = "compact_float")]
    min: f64,
    #[serde(with = "compact_float")]
    max: f64,
    #[serde(with = "compact_float")]
    mean: f64,
}

impl SummaryStats {
    pub fn min(&self) -> f64 {
        self.min
    }
    pub fn max(&self) -> f64 {
        self.max
    }
    pub fn mean(&self) -> f64 {
        self.mean
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SnailStats {
    #[serde(rename = "assembly")]
    span: usize,
    #[serde(rename = "ATGC")]
    atgc: usize,
    #[serde(rename = "GC", with = "compact_float")]
    gc_proportion: f64,
    #[serde(rename = "N")]
    n: usize,
    #[serde(rename = "binned_GCs")]
    binned_gcs: Vec<SummaryStats>,
    #[serde(rename = "binned_Ns")]
    binned_ns: Vec<SummaryStats>,
    scaffolds: Vec<usize>,
    scaffold_count: usize,
    binned_scaffold_lengths: Vec<usize>,
    binned_scaffold_counts: Vec<usize>,
}

impl SnailStats {
    pub fn span(&self) -> usize {
        self.span
    }
    pub fn atgc(&self) -> usize {
        self.atgc
    }
    pub fn n(&self) -> usize {
        self.n
    }
    pub fn binned_gcs(&self) -> &Vec<SummaryStats> {
        &self.binned_gcs
    }
    pub fn binned_ns(&self) -> &Vec<SummaryStats> {
        &self.binned_ns
    }
    pub fn scaffolds(&self) -> &Vec<usize> {
        &self.scaffolds
    }
    pub fn scaffold_count(&self) -> usize {
        self.scaffold_count
    }
    pub fn binned_scaffold_lengths(&self) -> &Vec<usize> {
        &self.binned_scaffold_lengths
    }
    pub fn binned_scaffold_counts(&self) -> &Vec<usize> {
        &self.binned_scaffold_counts
    }
}

fn indexed_sort<T: Ord>(list: &[T]) -> Vec<usize> {
    let mut indices = (0..list.len()).collect::<Vec<_>>();
    indices.sort_by_key(|&i| &list[i]);
    indices.reverse();
    indices
}

pub fn snail_stats(
    length_values: &Vec<usize>,
    gc_values: &Vec<f64>,
    n_values: &Vec<f64>,
    ncount_values: &Vec<usize>,
    busco_values: &Vec<Vec<blobdir::BuscoGene>>,
    options: &cli::PlotOptions,
) -> SnailStats {
    let span = length_values.iter().sum();
    let n = ncount_values.iter().sum();
    let atgc = span - n;
    let segment = span / options.segments;
    let order = indexed_sort(&length_values);
    // TODO: check span > segments
    let mut position: usize = 0;
    let mut binned_gcs: Vec<SummaryStats> = vec![];
    let mut binned_ns: Vec<SummaryStats> = vec![];
    let mut scaffold_index: usize = 0;
    let mut scaffold_sum: usize = length_values[order[scaffold_index]];
    let mut binned_scaffold_lengths: Vec<usize> = vec![];
    let mut binned_scaffold_counts: Vec<usize> = vec![];
    for _ in 0..options.segments {
        position += segment;
        let mut gcs: Vec<f64> = vec![gc_values[order[scaffold_index]] * 100.0];
        let mut ns: Vec<f64> = vec![n_values[order[scaffold_index]] * 100.0];
        while scaffold_sum < position {
            scaffold_index += 1;
            scaffold_sum += length_values[order[scaffold_index]];
            gcs.push(gc_values[order[scaffold_index]] * 100.0);
            ns.push(n_values[order[scaffold_index]] * 100.0);
        }
        binned_scaffold_counts.push(scaffold_index + 1);
        binned_scaffold_lengths.push(length_values[order[scaffold_index]]);
        gcs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        binned_gcs.push(SummaryStats {
            min: gcs[0],
            max: gcs[gcs.len() - 1],
            mean: gcs.iter().sum::<f64>() / gcs.len() as f64,
        });
        ns.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        binned_ns.push(SummaryStats {
            min: ns[0],
            max: ns[ns.len() - 1],
            mean: ns.iter().sum::<f64>() / ns.len() as f64,
        });
    }
    SnailStats {
        span,
        atgc,
        gc_proportion: 100.0, // TODO: calculate this!
        n,
        binned_gcs,
        binned_ns,
        scaffolds: vec![length_values[order[0]]],
        scaffold_count: length_values.len(),
        binned_scaffold_lengths,
        binned_scaffold_counts,
    }
}

pub fn linear_scale(value: usize, domain: &[usize; 2], range: &[f64; 2]) -> f64 {
    let proportion = (value - domain[0]) as f64 / (domain[1] - domain[0]) as f64;
    (range[1] - range[0]) * proportion + range[0]
}

pub fn linear_scale_float(value: f64, domain: &[f64; 2], range: &[f64; 2]) -> f64 {
    let proportion = (value - domain[0]) / (domain[1] - domain[0]);
    (range[1] - range[0]) * proportion + range[0]
}

pub fn sqrt_scale(value: usize, domain: &[usize; 2], range: &[f64; 2]) -> f64 {
    let proportion = ((value - domain[0]) as f64).sqrt() / ((domain[1] - domain[0]) as f64).sqrt();
    (range[1] - range[0]) * proportion + range[0]
}

pub fn sqrt_scale_float(value: f64, domain: &[f64; 2], range: &[f64; 2]) -> f64 {
    let proportion = (value - domain[0]).sqrt() / (domain[1] - domain[0]).sqrt();
    (range[1] - range[0]) * proportion + range[0]
}

pub enum TickStatus {
    Major,
    Minor,
}

pub struct RadialTick {
    index: usize,
    offset: f64,
    angle: f64,
    label: Text,
    path: Path,
    midpoint: (f64, f64),
    status: TickStatus,
}

pub fn set_tick(
    index: usize,
    offset: f64,
    bin_count: usize,
    max_radians: f64,
    label: String,
    angle_domain: &[f64; 2],
    angle_range: &[f64; 2],
    tick_domain: &[usize; 2],
    tick_range: &[f64; 2],
    status: &TickStatus,
) -> RadialTick {
    let angle = linear_scale_float(index as f64 + offset, &angle_domain, &angle_range);

    let mut adjusted_tick_range = [tick_range[0], tick_range[1]];
    if offset > 0.0 {
        let segment = max_radians / 2.0 / bin_count as f64;
        let theta = PI - segment - PI / 2 as f64;
        let inradius = tick_range[0] as f64 * theta.sin();
        let diff = (0.5 - offset).abs();
        let diff_segment = segment * diff * 2.0;
        let adjusted_radius = inradius / diff_segment.cos();

        let outer_inradius = tick_range[1] as f64 * theta.sin();
        let outer_adjusted_radius = outer_inradius / diff_segment.cos();
        adjusted_tick_range = [adjusted_radius, outer_adjusted_radius];
    }

    let tick_size = match status {
        TickStatus::Major => 5,
        TickStatus::Minor => 3,
    };
    let tick_distances = [
        linear_scale(0, &tick_domain, &adjusted_tick_range),
        linear_scale(tick_size, &tick_domain, &adjusted_tick_range),
        linear_scale(
            tick_domain[1] - tick_size,
            &tick_domain,
            &adjusted_tick_range,
        ),
        linear_scale(tick_domain[1], &tick_domain, &adjusted_tick_range),
    ];
    let midpoint = polar2cartesian(&Vector2::new(
        (adjusted_tick_range[1] - adjusted_tick_range[0]) / 2.0 + adjusted_tick_range[0],
        angle,
    ));
    let tick_points = [
        polar2cartesian(&Vector2::new(tick_distances[0], angle)),
        polar2cartesian(&Vector2::new(tick_distances[1], angle)),
        polar2cartesian(&Vector2::new(tick_distances[2], angle)),
        polar2cartesian(&Vector2::new(tick_distances[3], angle)),
    ];
    let tick_path_data = Data::new()
        .move_to((tick_points[0][0], tick_points[0][1]))
        .line_to((tick_points[1][0], tick_points[1][1]))
        .move_to((tick_points[2][0], tick_points[2][1]))
        .line_to((tick_points[3][0], tick_points[3][1]));

    let path = Path::new()
        .set("stroke", "black")
        .set("fill", "none")
        .set("stroke-width", 3)
        .set("d", tick_path_data);
    let text = Text::new()
        .set("font-family", "Roboto, Open sans, sans-serif")
        .set("font-size", "16")
        .set("text-anchor", "middle")
        .set("dominant-baseline", "middle")
        .set("stroke", "none")
        .set("fill", "black")
        .set(
            "transform",
            format!(
                "translate({:?}, {:?}) rotate({:?})",
                midpoint[0],
                midpoint[1],
                (angle + PI / 2.0) * 180.0 / PI
            ),
        )
        .add(nodeText::new(label));

    RadialTick {
        index,
        offset,
        angle,
        label: text,
        path,
        midpoint: (midpoint[0], midpoint[1]),
        status: match status {
            TickStatus::Major => TickStatus::Major,
            TickStatus::Minor => TickStatus::Minor,
        },
    }
}

pub fn set_axis_ticks(
    bin_count: usize,
    tick_count: usize,
    status: TickStatus,
    max_radians: f64,
    radius: f64,
    outer_radius: f64,
) -> Vec<RadialTick> {
    let (divisor, remainder) = div_rem(bin_count, tick_count);
    let angle_domain = [0.0, bin_count as f64];
    let angle_range = [-PI / 2.0, max_radians - PI / 2.0];
    let tick_domain = [0, 24];
    let tick_range = [radius, outer_radius];
    let mut ticks: Vec<RadialTick> = vec![];
    if remainder == 0 {
        // if divisor > 1 {
        ticks.push(set_tick(
            0,
            0.0,
            bin_count,
            max_radians,
            String::from("0%"),
            &angle_domain,
            &angle_range,
            &tick_domain,
            &tick_range,
            &status,
        ));
        // }
        for i in (divisor..bin_count + 1).step_by(divisor) {
            let label = format!("{}", ((i) as f64 / bin_count as f64 * 100.0) as u64);
            ticks.push(set_tick(
                i,
                0.0,
                bin_count,
                max_radians,
                label,
                &angle_domain,
                &angle_range,
                &tick_domain,
                &tick_range,
                &status,
            ));
        }
    } else {
        let mut sum: f64 = 0.0;
        let step = divisor as f64 + remainder as f64 / tick_count as f64;
        ticks.push(set_tick(
            0,
            0.0,
            bin_count,
            max_radians,
            String::from("0%"),
            &angle_domain,
            &angle_range,
            &tick_domain,
            &tick_range,
            &status,
        ));
        while ticks.len() < tick_count + 1 {
            sum += step;
            let adj_sum = sum + 0.001;
            let index = adj_sum.floor() as usize;
            let label = format!("{}", (sum / bin_count as f64 * 100.0).round() as u64);
            ticks.push(set_tick(
                index,
                ((sum - index as f64).abs() * 1000.0).round() / 1000.0,
                bin_count,
                max_radians,
                label,
                &angle_domain,
                &angle_range,
                &tick_domain,
                &tick_range,
                &status,
            ))
        }
    }
    ticks
}

pub fn svg(snail_stats: &SnailStats, options: &cli::PlotOptions) -> () {
    let max_span: usize = snail_stats.span();
    let max_scaffold: usize = snail_stats.scaffolds()[0];
    let radius: f64 = 375.0;
    let outer_radius: f64 = 450.0;
    let bin_count = snail_stats.binned_scaffold_lengths().len();

    let max_radians: f64 = PI * 1.9999999 * snail_stats.span() as f64 / max_span as f64;
    let n50_index = (bin_count / 2) - 1;
    let n90_index = (9 * bin_count / 10) - 1;
    let major_tick_count = 10;
    let minor_tick_count = 50;
    let major_ticks = set_axis_ticks(
        bin_count,
        major_tick_count,
        TickStatus::Major,
        max_radians,
        radius,
        outer_radius,
    );
    let minor_ticks = set_axis_ticks(
        bin_count,
        minor_tick_count,
        TickStatus::Minor,
        max_radians,
        radius,
        outer_radius,
    );
    let scaled_n50 = sqrt_scale(
        snail_stats.binned_scaffold_lengths()[n50_index],
        &[0, max_scaffold],
        &[radius, 0.0],
    );
    let scaled_n90 = sqrt_scale(
        snail_stats.binned_scaffold_lengths()[n90_index],
        &[0, max_scaffold],
        &[radius, 0.0],
    );

    let mut polar_scaf_coords: Vec<Vec<f64>> = vec![];
    let mut polar_longest_coords: Vec<Vec<f64>> = vec![];
    let mut show_longest: bool = false;
    let mut polar_n50_coords: Vec<Vec<f64>> = vec![];
    let mut polar_n90_coords: Vec<Vec<f64>> = vec![];
    let polar_axis_coords: Vec<Vec<f64>> = vec![];
    let mut polar_gc_coords: Vec<Vec<f64>> = vec![];
    let mut polar_at_coords: Vec<Vec<f64>> = vec![];
    let mut polar_inner_n_coords: Vec<Vec<f64>> = vec![];
    let mut polar_outer_n_coords: Vec<Vec<f64>> = vec![];
    for i in 0..bin_count {
        // angle
        let angle = linear_scale(i + 1, &[0, bin_count], &[-PI / 2.0, max_radians - PI / 2.0]);

        //scaffold lengths
        let scaf_length_polar: Vec<f64> = vec![
            sqrt_scale(
                snail_stats.binned_scaffold_lengths()[i],
                &[0, max_scaffold],
                &[radius, 0.0],
            ),
            angle,
        ];
        polar_scaf_coords.push(scaf_length_polar);

        // gc
        let gc_stats = &snail_stats.binned_gcs()[i];
        let gc_prop_polar: Vec<f64> = vec![
            linear_scale_float(gc_stats.mean(), &[0.0, 100.0], &[radius, outer_radius]),
            angle,
        ];
        polar_gc_coords.push(gc_prop_polar);

        // at
        let at_prop_polar: Vec<f64> = vec![
            linear_scale_float(
                100.0 - gc_stats.mean(),
                &[0.0, 100.0],
                &[outer_radius, radius],
            ),
            angle,
        ];
        polar_at_coords.push(at_prop_polar);

        // n
        let n_stats = &snail_stats.binned_ns()[i];
        let n_prop_inner: Vec<f64> = vec![
            linear_scale_float(n_stats.mean() / 2.0, &[0.0, 100.0], &[radius, outer_radius]),
            angle,
        ];
        polar_inner_n_coords.push(n_prop_inner);
        let n_prop_outer: Vec<f64> = vec![
            linear_scale_float(n_stats.mean() / 2.0, &[0.0, 100.0], &[outer_radius, radius]),
            angle,
        ];
        polar_outer_n_coords.push(n_prop_outer);

        // longest scaffold
        if snail_stats.binned_scaffold_lengths()[i] == max_scaffold {
            let longest_polar: Vec<f64> = vec![0.0, angle];
            polar_longest_coords.push(longest_polar);
            show_longest = true;
        }

        // n50/n90
        if i <= n90_index {
            if i <= n50_index {
                let n50_polar: Vec<f64> = vec![scaled_n50, angle];
                polar_n50_coords.push(n50_polar);
            }
            let n90_polar: Vec<f64> = vec![scaled_n90, angle];
            polar_n90_coords.push(n90_polar);
        }
    }
    let scaf_length_data = polar_to_path(&polar_scaf_coords, radius, bin_count, max_radians);
    let gc_prop_data = polar_to_path(&polar_gc_coords, radius, bin_count, max_radians);
    let at_prop_data = polar_to_path(&polar_at_coords, outer_radius, bin_count, max_radians);
    let n_prop_inner_data = polar_to_path(&polar_inner_n_coords, radius, bin_count, max_radians);
    let n_prop_outer_data =
        polar_to_path(&polar_outer_n_coords, outer_radius, bin_count, max_radians);
    let longest_arc_data = polar_to_path(&polar_longest_coords, radius, bin_count, max_radians);
    let n50_arc_data = polar_to_path(&polar_n50_coords, radius, bin_count, max_radians);
    let n90_arc_data = polar_to_path(&polar_n90_coords, radius, bin_count, max_radians);
    let axis_arc_data = polar_to_path(&polar_axis_coords, radius, bin_count, max_radians);
    let outer_axis_arc_data =
        polar_to_path(&polar_axis_coords, outer_radius, bin_count, max_radians);
    let longest_arc_outline_data =
        polar_to_path(&polar_longest_coords, radius, bin_count, max_radians);
    let n50_arc_outline_data = polar_to_path(&polar_n50_coords, radius, bin_count, max_radians);

    let scaf_length_path = Path::new()
        .set("fill", "#999999")
        .set("stroke", "none")
        // .set("stroke-width", 3)
        .set("d", scaf_length_data);

    let gc_prop_path = Path::new()
        .set("fill", "#1f78b4")
        .set("stroke", "none")
        // .set("stroke-width", 3)
        .set("d", gc_prop_data);

    let at_prop_path = Path::new()
        .set("fill", "#a6cee3")
        .set("stroke", "none")
        // .set("stroke-width", 3)
        .set("d", at_prop_data);

    let n_prop_inner_path = Path::new()
        .set("fill", "#ffffff")
        .set("stroke", "none")
        // .set("stroke-width", 3)
        .set("d", n_prop_inner_data);

    let n_prop_outer_path = Path::new()
        .set("fill", "#ffffff")
        .set("stroke", "none")
        // .set("stroke-width", 3)
        .set("d", n_prop_outer_data);

    let longest_arc_path = if show_longest {
        Path::new()
            .set("fill", "#e31a1c")
            .set("stroke", "none")
            // .set("stroke-width", 3)
            .set("d", longest_arc_data)
    } else {
        Path::new()
    };
    let n50_arc_path = Path::new()
        .set("fill", "#ff7f00")
        .set("stroke", "none")
        // .set("stroke-width", 3)
        .set("d", n50_arc_data);
    let n90_arc_path = Path::new()
        .set("fill", "#fdbf6f")
        .set("stroke", "none")
        // .set("stroke-width", 3)
        .set("d", n90_arc_data);

    let n50_arc_outline_path = Path::new()
        .set("stroke", "#ff7f00")
        .set("fill", "none")
        .set("stroke-width", 3)
        .set("d", n50_arc_outline_data);
    let longest_arc_outline_path = Path::new()
        .set("stroke", "#e31a1c")
        .set("fill", "none")
        .set("stroke-width", 3)
        .set("d", longest_arc_outline_data);

    let inner = Path::new()
        .set("stroke", "black")
        .set("fill", "none")
        .set("stroke-width", 3)
        .set("d", axis_arc_data);

    let outer = Path::new()
        .set("stroke", "black")
        .set("fill", "none")
        .set("stroke-width", 3)
        .set("d", outer_axis_arc_data);

    let inner_axis = Line::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 3)
        .set("x1", 0.0)
        .set("y1", 0.0)
        .set("x2", 0.0)
        .set("y2", -radius);
    // let outer_axis = Line::new()
    //     .set("fill", "none")
    //     .set("stroke", "black")
    //     .set("stroke-width", 3)
    //     .set("x1", 0.0)
    //     .set("y1", -radius)
    //     .set("x2", 0.0)
    //     .set("y2", -outer_radius);

    let mut major_tick_group = Group::new();
    for tick in major_ticks {
        major_tick_group = major_tick_group.add(tick.path).add(tick.label)
    }
    let mut minor_tick_group = Group::new();
    for tick in minor_ticks {
        minor_tick_group = minor_tick_group.add(tick.path)
    }

    let group = Group::new()
        .set("transform", "translate(500, 500)")
        .add(scaf_length_path)
        .add(gc_prop_path)
        .add(at_prop_path)
        .add(n_prop_inner_path)
        .add(n_prop_outer_path)
        .add(longest_arc_path)
        .add(n50_arc_path)
        .add(n90_arc_path)
        .add(n50_arc_outline_path)
        .add(longest_arc_outline_path)
        .add(major_tick_group)
        .add(minor_tick_group)
        .add(inner_axis)
        // .add(outer_axis)
        .add(inner)
        .add(outer);

    let document = Document::new()
        .set("viewBox", (0, 0, 1000, 1000))
        .add(group);

    svg::save(options.output.as_str(), &document).unwrap();
}

fn polar_to_path(
    polar_coords: &Vec<Vec<f64>>,
    radius: f64,
    bin_count: usize,
    max_radians: f64,
) -> Data {
    let mut path_data = Data::new();

    let mut length = polar_coords.len();
    if length == 0 {
        length = bin_count;
    }

    for i in (0..length).rev() {
        let angle = if polar_coords.len() > 0 {
            polar_coords[i][1]
        } else {
            max_radians * (i + 1) as f64 / length as f64 - PI / 2.0
        };
        let polar_coord = Vector2::new(radius, angle);
        let cartesian_coord = polar2cartesian(&polar_coord);
        if i == length - 1 {
            path_data = path_data.move_to((cartesian_coord[0], cartesian_coord[1]))
        } else {
            path_data = path_data.line_to((cartesian_coord[0], cartesian_coord[1]))
        }
        if i == 0 {
            let final_polar_coord = Vector2::new(radius, -PI / 2.0);
            let final_cartesian_coord = polar2cartesian(&final_polar_coord);
            path_data = path_data.line_to((final_cartesian_coord[0], final_cartesian_coord[1]))
        }
    }

    for i in 0..polar_coords.len() {
        let polar_coord_end = Vector2::new(polar_coords[i][0], polar_coords[i][1]);
        let mut polar_coord_start = Vector2::new(0.0, 0.0);
        if i > 0 {
            polar_coord_start = Vector2::new(polar_coords[i][0], polar_coords[i - 1][1]);
        } else {
            polar_coord_start = Vector2::new(
                polar_coords[i][0],
                linear_scale(0, &[0, bin_count], &[-PI / 2.0, max_radians - PI / 2.0]),
            );
        };

        let cartesian_start = polar2cartesian(&polar_coord_start);
        let cartesian_end = polar2cartesian(&polar_coord_end);
        path_data = path_data
            .line_to((cartesian_start[0], cartesian_start[1]))
            .line_to((cartesian_end[0], cartesian_end[1]));
        // }
    }
    if polar_coords.len() > 0 {
        path_data = path_data.close();
    }
    path_data
}
