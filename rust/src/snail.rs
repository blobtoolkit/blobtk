use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;

use cli::PlotOptions;
use coord_transforms::d2::polar2cartesian;
use coord_transforms::prelude::*;
use num_integer::div_rem;
use rust_decimal::prelude::*;
use serde;
use serde::{Deserialize, Serialize};
use svg::node::element::path::Data;
use svg::node::element::{Circle, Group, Line, Path, Rectangle, Text};
use svg::node::Text as nodeText;
use svg::Document;
use titlecase::titlecase;

use crate::blobdir::{self, BuscoGene};
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
    id: String,
    #[serde(rename = "assembly")]
    span: usize,
    #[serde(rename = "ATGC")]
    atgc: usize,
    #[serde(rename = "GC", with = "compact_float")]
    gc_proportion: f64,
    #[serde(rename = "AT", with = "compact_float")]
    at_proportion: f64,
    n_proportion: f64,
    #[serde(rename = "N")]
    n: usize,
    #[serde(rename = "binned_GCs")]
    binned_gcs: Vec<SummaryStats>,
    #[serde(rename = "binned_Ns")]
    binned_ns: Vec<SummaryStats>,
    busco_complete: usize,
    busco_fragmented: usize,
    busco_duplicated: usize,
    busco_total: usize,
    busco_lineage: String,
    record_type: String,
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
    pub fn busco_complete(&self) -> usize {
        self.busco_complete
    }
    pub fn busco_fragmented(&self) -> usize {
        self.busco_fragmented
    }
    pub fn busco_duplicated(&self) -> usize {
        self.busco_duplicated
    }
    pub fn busco_total(&self) -> usize {
        self.busco_total
    }
    pub fn busco_lineage(&self) -> &String {
        &self.busco_lineage
    }
    pub fn record_type(&self) -> &String {
        &self.record_type
    }
}

fn indexed_sort<T: Ord>(list: &[T]) -> Vec<usize> {
    let mut indices = (0..list.len()).collect::<Vec<_>>();
    indices.sort_by_key(|&i| &list[i]);
    indices.reverse();
    indices
}

fn count_buscos(
    busco_values: &Vec<BuscoGene>,
    busco_frag: &mut HashSet<String>,
    busco_list: &mut HashSet<String>,
    busco_dup: &mut HashSet<String>,
) {
    for busco in busco_values.clone().into_iter() {
        let busco_id = busco.id;
        if busco.status == "Fragmented".to_string() {
            busco_frag.insert(busco_id.clone());
        }
        if busco_list.contains(&busco_id) {
            busco_dup.insert(busco_id.clone());
        }
        busco_list.insert(busco_id);
    }
}

pub fn snail_stats(
    length_values: &Vec<usize>,
    gc_values: &Vec<f64>,
    n_vals: &Option<Vec<f64>>,
    ncount_values: &Vec<usize>,
    busco_values: &Vec<Vec<blobdir::BuscoGene>>,
    busco_total: usize,
    busco_lineage: String,
    id: String,
    record_type: String,
    options: &cli::PlotOptions,
) -> SnailStats {
    let span = length_values.iter().sum();
    let n = ncount_values.iter().sum();
    let mut new_vals = vec![];
    let n_values = match n_vals {
        Some(vals) => vals,
        None => {
            for (i, length) in length_values.iter().enumerate() {
                new_vals.push(ncount_values[i] as f64 / length.clone() as f64);
            }
            &new_vals
        }
    };
    let atgc = span - n;
    let segment = span / options.segments;
    let order = indexed_sort(&length_values);
    // TODO: check span > segments
    let mut position: usize = 0;
    let mut binned_gcs: Vec<SummaryStats> = vec![];
    let mut binned_ns: Vec<SummaryStats> = vec![];
    let mut busco_list = HashSet::new();
    let mut busco_frag = HashSet::new();
    let mut busco_dup = HashSet::new();
    let mut scaffold_index: usize = 0;
    let mut scaffold_sum: usize = length_values[order[scaffold_index]];
    let mut gc_span = gc_values[order[scaffold_index]]
        * ((length_values[order[scaffold_index]] - ncount_values[order[scaffold_index]]) as f64);
    let mut at_span = (1.0 - gc_values[order[scaffold_index]])
        * ((length_values[order[scaffold_index]] - ncount_values[order[scaffold_index]]) as f64);
    let mut n_span = ncount_values[order[scaffold_index]];
    count_buscos(
        &busco_values[order[scaffold_index]],
        &mut busco_frag,
        &mut busco_list,
        &mut busco_dup,
    );

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

            gc_span += gc_values[order[scaffold_index]]
                * ((length_values[order[scaffold_index]] - ncount_values[order[scaffold_index]])
                    as f64);
            at_span += (1.0 - gc_values[order[scaffold_index]])
                * ((length_values[order[scaffold_index]] - ncount_values[order[scaffold_index]])
                    as f64);
            n_span += ncount_values[order[scaffold_index]];
            count_buscos(
                &busco_values[order[scaffold_index]],
                &mut busco_frag,
                &mut busco_list,
                &mut busco_dup,
            );
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
        gc_proportion: gc_span / span as f64,
        at_proportion: at_span / span as f64,
        n_proportion: n_span as f64 / span as f64,
        n,
        binned_gcs,
        binned_ns,
        scaffolds: vec![length_values[order[0]]],
        scaffold_count: length_values.len(),
        busco_complete: busco_list.len(),
        busco_duplicated: busco_dup.len(),
        busco_fragmented: busco_frag.len(),
        busco_total: busco_total,
        busco_lineage,
        binned_scaffold_lengths,
        binned_scaffold_counts,
        id,
        record_type,
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

pub fn log_scale(value: usize, domain: &[usize; 2], range: &[f64; 2]) -> f64 {
    let proportion =
        ((value - domain[0]) as f64).log10() / ((domain[1] - domain[0]) as f64).log10();
    (range[1] - range[0]) * proportion + range[0]
}

pub fn log_scale_float(value: f64, domain: &[f64; 2], range: &[f64; 2]) -> f64 {
    let proportion = (value - domain[0]).log10() / (domain[1] - domain[0]).log10();
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

pub struct TickOptions {
    font_size: f64,
    font_color: String,
    show_secondary_tick: bool,
    status: TickStatus,
}

impl Default for TickOptions {
    fn default() -> TickOptions {
        TickOptions {
            font_size: 20.0,
            font_color: "black".to_string(),
            show_secondary_tick: false,
            status: TickStatus::Major,
        }
    }
}

pub struct RadialTick {
    index: usize,
    offset: f64,
    angle: f64,
    label: Text,
    path: Path,
    outer_label: Text,
    midpoint: (f64, f64),
    status: TickStatus,
}

pub struct Tick {
    label: Text,
    path: Path,
    position: f64,
    status: TickStatus,
}

pub fn path_axis_major(path_data: Data, color: Option<&str>) -> Path {
    let col = color.unwrap_or("black");
    Path::new()
        .set("stroke", col)
        .set("fill", "none")
        .set("stroke-width", 3)
        .set("d", path_data)
}

pub fn path_axis_minor(path_data: Data, color: Option<&str>) -> Path {
    let col = color.unwrap_or("black");
    Path::new()
        .set("stroke", col)
        .set("fill", "none")
        .set("stroke-width", 1)
        .set("d", path_data)
}

pub fn path_gridline_major(path_data: Data, color: Option<&str>) -> Path {
    let col = color.unwrap_or("black");
    Path::new()
        .set("stroke", col)
        .set("fill", "none")
        .set("stroke-width", 2)
        .set("d", path_data)
}

pub fn path_gridline_minor(path_data: Data, color: Option<&str>) -> Path {
    let col = color.unwrap_or("black");
    Path::new()
        .set("stroke", col)
        .set("fill", "none")
        .set("stroke-width", 1)
        .set("stroke-dasharray", "5, 5")
        .set("d", path_data)
}

pub fn path_filled(path_data: Data, color: Option<&str>) -> Path {
    let col = color.unwrap_or("black");
    Path::new()
        .set("stroke", "none")
        .set("fill", col)
        .set("stroke-width", 0)
        .set("d", path_data)
}

pub fn path_open(path_data: Data, color: Option<&str>, weight: Option<f64>) -> Path {
    let col = color.unwrap_or("black");
    let stroke_width = weight.unwrap_or(3.0);
    Path::new()
        .set("stroke", col)
        .set("fill", "none")
        .set("stroke-width", stroke_width)
        .set("d", path_data)
}

pub fn path_partial(path_data: Data, color: Option<&str>, weight: Option<f64>) -> Path {
    let col = color.unwrap_or("black");
    let stroke_width = weight.unwrap_or(1.0);
    Path::new()
        .set("stroke", col)
        .set("stroke-opacity", 0.4)
        .set("fill", col)
        .set("fill-opacity", 0.2)
        .set("stroke-width", stroke_width)
        .set("d", path_data)
}

pub fn set_tick_circular(
    index: usize,
    offset: f64,
    bin_count: usize,
    max_radians: f64,
    label: String,
    outer_label: String,
    angle_domain: &[f64; 2],
    angle_range: &[f64; 2],
    tick_domain: &[usize; 2],
    tick_range: &[f64; 2],
    status: &TickStatus,
    options: &TickOptions,
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
    let outer_point = polar2cartesian(&Vector2::new(tick_distances[3] + 4.0, angle));
    let tick_path_data = if options.show_secondary_tick {
        Data::new()
            .move_to((tick_points[0][0], tick_points[0][1]))
            .line_to((tick_points[1][0], tick_points[1][1]))
            .move_to((tick_points[2][0], tick_points[2][1]))
            .line_to((tick_points[3][0], tick_points[3][1]))
    } else {
        Data::new()
            .move_to((tick_points[0][0], tick_points[0][1]))
            .line_to((tick_points[1][0], tick_points[1][1]))
    };
    let path = match status {
        TickStatus::Major => path_axis_major(tick_path_data, None),
        TickStatus::Minor => path_axis_minor(tick_path_data, None),
    };
    let text = if label == "100".to_string() && angle > 1.4 * PI {
        Text::new()
    } else {
        Text::new()
            .set("font-family", "Roboto, Open sans, sans-serif")
            .set("font-size", options.font_size.clone())
            .set("text-anchor", "middle")
            .set("dominant-baseline", "middle")
            .set("stroke", "none")
            .set("fill", options.font_color.clone())
            .set(
                "transform",
                format!(
                    "translate({:?}, {:?}) rotate({:?})",
                    midpoint[0],
                    midpoint[1],
                    (angle + PI / 2.0) * 180.0 / PI
                ),
            )
            .add(nodeText::new(label))
    };
    let outer_text = match status {
        TickStatus::Major => Text::new()
            .set("font-family", "Roboto, Open sans, sans-serif")
            .set("font-size", "20")
            .set("text-anchor", "middle")
            .set("dominant-baseline", "bottom")
            .set("stroke", "none")
            .set("fill", "black")
            .set(
                "transform",
                format!(
                    "translate({:?}, {:?}) rotate({:?})",
                    outer_point[0],
                    outer_point[1],
                    (angle + PI / 2.0) * 180.0 / PI
                ),
            )
            .add(nodeText::new(outer_label)),
        TickStatus::Minor => Text::new(),
    };

    RadialTick {
        index,
        offset,
        angle,
        label: text,
        outer_label: outer_text,
        path,
        midpoint: (midpoint[0], midpoint[1]),
        status: match status {
            TickStatus::Major => TickStatus::Major,
            TickStatus::Minor => TickStatus::Minor,
        },
    }
}

pub fn set_axis_ticks_circular(
    bin_count: usize,
    tick_count: usize,
    status: TickStatus,
    max_radians: f64,
    radius: f64,
    outer_radius: f64,
    span: usize,
    options: TickOptions,
) -> Vec<RadialTick> {
    let (divisor, remainder) = div_rem(bin_count, tick_count);
    let angle_domain = [0.0, bin_count as f64];
    let angle_range = [-PI / 2.0, max_radians - PI / 2.0];
    let tick_domain = [0, 24];
    let tick_range = [radius, outer_radius];
    let mut ticks: Vec<RadialTick> = vec![];
    if remainder == 0 {
        // if divisor > 1 {
        ticks.push(set_tick_circular(
            0,
            0.0,
            bin_count,
            max_radians,
            String::from("0%"),
            String::new(),
            &angle_domain,
            &angle_range,
            &tick_domain,
            &tick_range,
            &status,
            &options,
        ));
        // }
        for i in (divisor..bin_count + 1).step_by(divisor) {
            let label = format!("{}", ((i) as f64 / bin_count as f64 * 100.0) as u64);
            let outer_label = format_si(&(span as f64 / tick_count as f64 * ticks.len() as f64), 3);
            ticks.push(set_tick_circular(
                i,
                0.0,
                bin_count,
                max_radians,
                label,
                outer_label,
                &angle_domain,
                &angle_range,
                &tick_domain,
                &tick_range,
                &status,
                &options,
            ));
        }
    } else {
        let mut sum: f64 = 0.0;
        let step = divisor as f64 + remainder as f64 / tick_count as f64;
        ticks.push(set_tick_circular(
            0,
            0.0,
            bin_count,
            max_radians,
            String::from("0%"),
            String::new(),
            &angle_domain,
            &angle_range,
            &tick_domain,
            &tick_range,
            &status,
            &options,
        ));
        while ticks.len() < tick_count + 1 {
            sum += step;
            let adj_sum = sum + 0.001;
            let index = adj_sum.floor() as usize;
            let label = format!("{}", (sum / bin_count as f64 * 100.0).round() as u64);
            let outer_label = format_si(&(span as f64 / tick_count as f64 * ticks.len() as f64), 3);
            ticks.push(set_tick_circular(
                index,
                ((sum - index as f64).abs() * 1000.0).round() / 1000.0,
                bin_count,
                max_radians,
                label,
                outer_label,
                &angle_domain,
                &angle_range,
                &tick_domain,
                &tick_range,
                &status,
                &options,
            ))
        }
    }
    ticks
}

pub fn format_si(value: &f64, digits: u32) -> String {
    fn set_suffix(thousands: i8) -> String {
        const POSITIVE: [&str; 9] = ["", "k", "M", "G", "T", "P", "E", "Z", "Y"];
        const NEGATIVE: [&str; 9] = ["", "m", "μ", "p", "n", "f", "a", "z", "y"];
        let suffix = if thousands < 0 && thousands >= -9 {
            NEGATIVE[(thousands * -1) as usize]
        } else if thousands >= 0 && thousands <= 9 {
            POSITIVE[thousands as usize]
        } else {
            ""
        };
        suffix.to_string()
    }

    let magnitude = (value.clone()).log10() as i8;
    let thousands = magnitude / 3;
    let prefix = if thousands < 0 {
        value.clone()
    } else {
        value.clone() / 10u32.pow(3 * thousands as u32) as f64
    };
    let d = Decimal::from_f64_retain(prefix).unwrap();
    let rounded = d
        .round_sf_with_strategy(digits, RoundingStrategy::MidpointAwayFromZero)
        .unwrap()
        .normalize()
        .to_string();

    let suffix = set_suffix(thousands);
    format!("{}{}", rounded, suffix)
}

pub fn set_tick(
    value: f64,
    label: String,
    domain: &[f64; 2],
    range: &[f64; 2],
    status: &TickStatus,
) -> Tick {
    let offset = sqrt_scale_float(value, &domain, &range);
    let path = match status {
        TickStatus::Major => path_axis_major(
            Data::new().move_to((-10, offset)).line_to((0, offset)),
            None,
        ),
        TickStatus::Minor => {
            path_axis_minor(Data::new().move_to((-5, offset)).line_to((0, offset)), None)
        }
    };
    let text = match status {
        TickStatus::Major => Text::new()
            .set("font-family", "Roboto, Open sans, sans-serif")
            .set("font-size", "20")
            .set("text-anchor", "end")
            .set("dominant-baseline", "middle")
            .set("stroke", "none")
            .set("fill", "black")
            .set("transform", format!("translate({:?}, {:?})", -15, offset,))
            .add(nodeText::new(label)),
        TickStatus::Minor => Text::new(),
    };

    Tick {
        label: text,
        path,
        position: offset,
        status: match status {
            TickStatus::Major => TickStatus::Major,
            TickStatus::Minor => TickStatus::Minor,
        },
    }
}

pub fn set_axis_ticks(
    max_value: &f64,
    min_value: &f64,
    status: &TickStatus,
    dimension: &f64,
) -> Vec<Tick> {
    let range = [-dimension.clone(), 0.0];
    let domain = [min_value.clone(), max_value.clone()];

    let mut power = 0;
    let mut min_val = min_value.clone();
    while min_val > 1.0 {
        power += 1;
        min_val /= 10.0;
    }
    let mut i = 10u32.pow(power) as f64;

    let mut ticks: Vec<Tick> = vec![];
    match status {
        TickStatus::Major => {
            let mut i = 10u32.pow(power) as f64;
            while i <= max_value.clone() {
                let label = if i > min_value.clone() {
                    format_si(&i, 3)
                } else {
                    String::new()
                };
                ticks.push(set_tick(i, label, &domain, &range, &status));
                i = i * 10.0;
            }
        }
        TickStatus::Minor => {
            let mut i = 10u32.pow(power - 1) as f64;
            while i <= max_value.clone() {
                let mut j = i * 2.0;
                while j < i * 10.0 && j <= max_value.clone() {
                    if &(j as f64) >= min_value {
                        ticks.push(set_tick(j, String::new(), &domain, &range, &status));
                    }
                    j = j + i;
                }
                ticks.push(set_tick(i, String::new(), &domain, &range, &status));
                i = i * 10.0;
            }
        }
    }
    ticks
}

pub enum LegendShape {
    Rect,
    Circumference,
    Radius,
}

pub fn legend(
    title: String,
    entries: Vec<(String, String, LegendShape)>,
    subtitle: Option<String>,
    columns: u8,
) -> Group {
    let title_text = Text::new()
        .set("font-family", "Roboto, Open sans, sans-serif")
        .set("font-size", "24")
        .set("text-anchor", "start")
        .set("dominant-baseline", "bottom")
        .set("stroke", "none")
        .set("fill", "black")
        .add(nodeText::new(title));
    let mut group = Group::new().add(title_text);
    let cell = 18;
    let gap = 8;
    let mut offset_y = 0;
    let mut offset_x = -175;
    let per_column = entries.len() / columns as usize;
    for (i, entry) in entries.iter().enumerate() {
        if i % per_column == 0 {
            offset_x += 175;
            offset_y = gap / 2;
        }
        let shape = match entry.2 {
            LegendShape::Rect => Group::new().add(
                Rectangle::new()
                    .set("stroke", "black")
                    .set("stroke-width", 2)
                    .set("fill", entry.clone().1.clone())
                    .set("x", 0)
                    .set("y", 6)
                    .set("height", cell)
                    .set("width", cell),
            ),
            LegendShape::Circumference => Group::new()
                .add(
                    Circle::new()
                        .set("stroke", "black")
                        .set("stroke-width", 2)
                        .set("fill", entry.clone().1.clone())
                        .set("cx", cell / 2)
                        .set("cy", 6 + cell / 2)
                        .set("r", cell / 2),
                )
                .add(
                    Line::new()
                        .set("fill", "none")
                        .set("stroke", "black")
                        .set("stroke-width", 1)
                        .set("x1", cell / 2)
                        .set("y1", 6 + cell / 2)
                        .set("x2", cell / 2)
                        .set("y2", 6),
                ),
            LegendShape::Radius => Group::new()
                .add(
                    Circle::new()
                        .set("stroke", "black")
                        .set("stroke-width", 1)
                        .set("fill", entry.clone().1.clone())
                        .set("cx", cell / 2)
                        .set("cy", 6 + cell / 2)
                        .set("r", cell / 2),
                )
                .add(
                    Line::new()
                        .set("fill", "none")
                        .set("stroke", "black")
                        .set("stroke-width", 2)
                        .set("x1", cell / 2)
                        .set("y1", 6 + cell / 2)
                        .set("x2", cell / 2)
                        .set("y2", 6),
                ),
        };
        let entry_text = Text::new()
            .set("font-family", "Roboto, Open sans, sans-serif")
            .set("font-size", cell)
            .set("text-anchor", "start")
            .set("dominant-baseline", "bottom")
            .set("stroke", "none")
            .set("fill", "black")
            .set("x", cell + gap)
            .set("y", cell + gap / 2)
            .add(nodeText::new(&entry.clone().0));
        let entry_group = Group::new()
            .set(
                "transform",
                format!("translate({}, {})", offset_x, offset_y),
            )
            .add(shape)
            .add(entry_text);
        group = group.add(entry_group);
        offset_y = offset_y + cell + gap;
    }
    match subtitle {
        None => (),
        Some(subtitle_string) => {
            let subtitle_text = Text::new()
                .set("font-family", "Roboto, Open sans, sans-serif")
                .set("font-size", "18")
                .set("text-anchor", "start")
                .set("dominant-baseline", "bottom")
                .set("stroke", "none")
                .set("fill", "black")
                .set("transform", "translate(100, 0)")
                .add(nodeText::new(subtitle_string));
            group = group.add(subtitle_text);
        }
    }

    group
}

pub fn scaffold_stats_legend(snail_stats: &SnailStats, options: &cli::PlotOptions) -> Group {
    let mut entries = vec![];
    let scaffold_count = format_si(&(snail_stats.scaffold_count() as f64), 3);
    let scaffold_length = format_si(&(snail_stats.span() as f64), 3);
    let longest_scaffold = format_si(&(snail_stats.scaffolds()[0] as f64), 3);
    let n50_bin = (options.segments / 2) - 1;
    let n90_bin = (options.segments * 9 / 10) - 1;
    let n50_length = format_si(&(snail_stats.binned_scaffold_lengths()[n50_bin] as f64), 3);
    let n90_length = format_si(&(snail_stats.binned_scaffold_lengths()[n90_bin] as f64), 3);
    let record = snail_stats.record_type();
    entries.push((
        format!("Log10 {} count (total {})", record, scaffold_count),
        "#dddddd".to_string(),
        LegendShape::Rect,
    ));
    entries.push((
        format!("{} length (total {})", titlecase(record), scaffold_length),
        "#999999".to_string(),
        LegendShape::Rect,
    ));
    entries.push((
        format!("Longest {} ({})", record, longest_scaffold),
        "#e31a1c".to_string(),
        LegendShape::Rect,
    ));
    entries.push((
        format!("N50 length ({})", n50_length),
        "#ff7f00".to_string(),
        LegendShape::Rect,
    ));
    entries.push((
        format!("N90 length ({})", n90_length),
        "#fdbf6f".to_string(),
        LegendShape::Rect,
    ));

    let title = format!("{} statistics", titlecase(record));
    legend(title, entries, None, 1)
}

pub fn composition_stats_legend(snail_stats: &SnailStats, options: &cli::PlotOptions) -> Group {
    let mut entries = vec![];
    let gc_prop = format_si(&(snail_stats.gc_proportion as f64 * 100.0), 3);
    let at_prop = format_si(&(snail_stats.at_proportion as f64 * 100.0), 3);
    let n_prop = format_si(&(snail_stats.n_proportion as f64 * 100.0), 3);
    entries.push((
        format!("GC ({}%)", gc_prop),
        "#1f78b4".to_string(),
        LegendShape::Rect,
    ));
    entries.push((
        format!("AT ({}%)", at_prop),
        "#a6cee3".to_string(),
        LegendShape::Rect,
    ));
    entries.push((
        format!("N ({}%)", n_prop),
        "#ffffff".to_string(),
        LegendShape::Rect,
    ));

    let title = "Composition".to_string();
    legend(title, entries, None, 1)
}

pub fn scale_stats_legend(snail_stats: &SnailStats, options: &cli::PlotOptions) -> Group {
    let mut entries = vec![];
    let max_span = match options.max_span {
        Some(span) => span,
        None => snail_stats.span(),
    };
    let max_scaffold = match options.max_scaffold {
        Some(scaffold_length) => scaffold_length,
        None => snail_stats.scaffolds()[0],
    };
    let circ_prop = format_si(&(max_span as f64), 3);
    let rad_prop = format_si(&(max_scaffold as f64), 3);
    entries.push((
        format!("{}", circ_prop),
        "#ffffff".to_string(),
        LegendShape::Circumference,
    ));
    entries.push((
        format!("{}", rad_prop),
        "#ffffff".to_string(),
        LegendShape::Radius,
    ));

    let title = "Scale".to_string();
    legend(title, entries, None, 1)
}

pub fn dataset_name_legend(snail_stats: &SnailStats, options: &cli::PlotOptions) -> Group {
    let entries = vec![];

    let title = format!("Dataset: {}", snail_stats.id);
    legend(title, entries, None, 1)
}

pub fn busco_stats_legend(snail_stats: &SnailStats, options: &cli::PlotOptions) -> Group {
    let mut entries = vec![];
    let comp_prop = format_si(
        &(snail_stats.busco_complete as f64 / snail_stats.busco_total as f64 * 100.0),
        3,
    );
    let dup_prop = format_si(
        &(snail_stats.busco_duplicated as f64 / snail_stats.busco_total as f64 * 100.0),
        3,
    );
    let frag_prop = format_si(
        &(snail_stats.busco_fragmented as f64 / snail_stats.busco_total as f64 * 100.0),
        3,
    );
    let missing_prop = format_si(
        &((snail_stats.busco_total - snail_stats.busco_complete) as f64
            / snail_stats.busco_total as f64
            * 100.0),
        3,
    );
    let subtitle = format!(
        "{} ({})",
        snail_stats.busco_lineage,
        snail_stats.busco_total()
    );
    entries.push((
        format!("Comp. ({}%)", comp_prop),
        "#33a02c".to_string(),
        LegendShape::Rect,
    ));
    entries.push((
        format!("Dupl. ({}%)", dup_prop),
        "#20641b".to_string(),
        LegendShape::Rect,
    ));
    entries.push((
        format!("Frag. ({}%)", frag_prop),
        "#a3e27f".to_string(),
        LegendShape::Rect,
    ));
    entries.push((
        format!("Missing ({}%)", missing_prop),
        "#ffffff".to_string(),
        LegendShape::Rect,
    ));

    let title = "BUSCO".to_string();
    legend(title, entries, Some(subtitle), 2)
}

pub fn svg(snail_stats: &SnailStats, options: &cli::PlotOptions) -> () {
    let max_span = match options.max_span {
        Some(span) => span,
        None => snail_stats.span(),
    };
    let max_scaffold = match options.max_scaffold {
        Some(scaffold_length) => scaffold_length,
        None => snail_stats.scaffolds()[0],
    };
    let radius: f64 = 375.0;
    let outer_radius: f64 = 450.0;
    let bin_count = snail_stats.binned_scaffold_lengths().len();

    let max_radians: f64 = PI * 1.9999999 * snail_stats.span() as f64 / max_span as f64;
    let n50_index = (bin_count / 2) - 1;
    let n90_index = (9 * bin_count / 10) - 1;
    let major_tick_count = 10;
    let minor_tick_count = 50;
    let major_ticks = set_axis_ticks_circular(
        bin_count,
        major_tick_count,
        TickStatus::Major,
        max_radians,
        radius,
        outer_radius,
        snail_stats.span(),
        TickOptions {
            show_secondary_tick: true,
            ..Default::default()
        },
    );
    let minor_ticks = set_axis_ticks_circular(
        bin_count,
        minor_tick_count,
        TickStatus::Minor,
        max_radians,
        radius,
        outer_radius,
        snail_stats.span(),
        TickOptions {
            show_secondary_tick: true,
            ..Default::default()
        },
    );
    let min_length_tick = snail_stats.binned_scaffold_lengths()[bin_count - 1];
    let major_length_ticks = set_axis_ticks(
        &(max_scaffold as f64),
        &(min_length_tick as f64),
        &TickStatus::Major,
        &radius,
    );
    let minor_length_ticks = set_axis_ticks(
        &(max_scaffold as f64),
        &(min_length_tick as f64),
        &TickStatus::Minor,
        &radius,
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
    let mut polar_count_coords: Vec<Vec<f64>> = vec![];
    let mut polar_longest_coords: Vec<Vec<f64>> = vec![];
    let mut show_longest: bool = false;
    let mut polar_n50_coords: Vec<Vec<f64>> = vec![];
    let mut polar_n90_coords: Vec<Vec<f64>> = vec![];
    let polar_axis_coords: Vec<Vec<f64>> = vec![];
    let mut polar_gc_coords: Vec<Vec<f64>> = vec![];
    let mut polar_gc_max_coords: Vec<Vec<f64>> = vec![];
    let mut polar_gc_min_coords: Vec<Vec<f64>> = vec![];
    let mut polar_at_coords: Vec<Vec<f64>> = vec![];
    let mut polar_inner_n_coords: Vec<Vec<f64>> = vec![];
    let mut polar_outer_n_coords: Vec<Vec<f64>> = vec![];
    let mut polar_inner_n_max_coords: Vec<Vec<f64>> = vec![];
    let mut polar_outer_n_max_coords: Vec<Vec<f64>> = vec![];
    let scaf_count_domain = [0, 10000000000];
    let scaf_count_range = [0.0, radius];
    for i in 0..bin_count {
        // angle
        let angle = linear_scale(i + 1, &[0, bin_count], &[-PI / 2.0, max_radians - PI / 2.0]);

        // scaffold lengths
        let scaf_length_polar: Vec<f64> = vec![
            sqrt_scale(
                snail_stats.binned_scaffold_lengths()[i],
                &[0, max_scaffold],
                &[radius, 0.0],
            ),
            angle,
        ];
        polar_scaf_coords.push(scaf_length_polar);

        // scaffold_counts
        let scaf_count_polar: Vec<f64> = vec![
            log_scale(
                snail_stats.binned_scaffold_counts()[i],
                &scaf_count_domain,
                &scaf_count_range,
            ),
            angle,
        ];
        polar_count_coords.push(scaf_count_polar);

        // gc
        let gc_stats = &snail_stats.binned_gcs()[i];
        let gc_prop_polar: Vec<f64> = vec![
            linear_scale_float(gc_stats.mean(), &[0.0, 100.0], &[radius, outer_radius]),
            angle,
        ];
        polar_gc_coords.push(gc_prop_polar);
        let gc_prop_max_polar: Vec<f64> = vec![
            linear_scale_float(gc_stats.max(), &[0.0, 100.0], &[radius, outer_radius]),
            angle,
        ];
        polar_gc_max_coords.push(gc_prop_max_polar);
        let gc_prop_min_polar: Vec<f64> = vec![
            linear_scale_float(gc_stats.min(), &[0.0, 100.0], &[radius, outer_radius]),
            angle,
        ];
        polar_gc_min_coords.push(gc_prop_min_polar);

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
        let n_prop_max_inner: Vec<f64> = vec![
            linear_scale_float(n_stats.max() / 2.0, &[0.0, 100.0], &[radius, outer_radius]),
            angle,
        ];
        polar_inner_n_max_coords.push(n_prop_max_inner);
        let n_prop_outer: Vec<f64> = vec![
            linear_scale_float(n_stats.mean() / 2.0, &[0.0, 100.0], &[outer_radius, radius]),
            angle,
        ];
        polar_outer_n_coords.push(n_prop_outer);
        let n_prop_max_outer: Vec<f64> = vec![
            linear_scale_float(n_stats.max() / 2.0, &[0.0, 100.0], &[outer_radius, radius]),
            angle,
        ];
        polar_outer_n_max_coords.push(n_prop_max_outer);

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
    let scaf_count_data = polar_to_path(&polar_count_coords, 0.0, bin_count, max_radians);
    let gc_prop_data = polar_to_path(&polar_gc_coords, radius, bin_count, max_radians);
    let gc_prop_max_data = polar_to_path_bounded(
        &polar_gc_max_coords,
        &polar_gc_coords,
        bin_count,
        max_radians,
    );
    let gc_prop_min_data = polar_to_path_bounded(
        &polar_gc_min_coords,
        &polar_gc_coords,
        bin_count,
        max_radians,
    );
    let at_prop_data = polar_to_path(&polar_at_coords, outer_radius, bin_count, max_radians);
    let n_prop_inner_data = polar_to_path(&polar_inner_n_coords, radius, bin_count, max_radians);
    let n_prop_outer_data =
        polar_to_path(&polar_outer_n_coords, outer_radius, bin_count, max_radians);
    let n_prop_inner_max_data =
        polar_to_path(&polar_inner_n_max_coords, radius, bin_count, max_radians);
    let n_prop_outer_max_data = polar_to_path(
        &polar_outer_n_max_coords,
        outer_radius,
        bin_count,
        max_radians,
    );
    let longest_arc_data = polar_to_path(&polar_longest_coords, radius, bin_count, max_radians);
    let n50_arc_data = polar_to_path(&polar_n50_coords, radius, bin_count, max_radians);
    let n90_arc_data = polar_to_path(&polar_n90_coords, radius, bin_count, max_radians);
    let axis_arc_data = polar_to_path(&polar_axis_coords, radius, bin_count, max_radians);
    let outer_axis_arc_data =
        polar_to_path(&polar_axis_coords, outer_radius, bin_count, max_radians);
    let longest_arc_outline_data =
        polar_to_path(&polar_longest_coords, radius, bin_count, max_radians);
    let n50_arc_outline_data = polar_to_path(&polar_n50_coords, radius, bin_count, max_radians);

    let scaf_length_path = path_filled(scaf_length_data, Some("#999999"));
    let scaf_count_path = path_filled(scaf_count_data, Some("#dddddd"));
    let gc_prop_path = path_filled(gc_prop_data, Some("#1f78b4"));
    let gc_prop_max_path = path_partial(gc_prop_max_data, Some("#1f78b4"), None);
    let gc_prop_min_path = path_partial(gc_prop_min_data, Some("#a6cee3"), None);
    let at_prop_path = path_filled(at_prop_data, Some("#a6cee3"));
    let n_prop_inner_path = path_filled(n_prop_inner_data, Some("#ffffff"));
    let n_prop_outer_path = path_filled(n_prop_outer_data, Some("#ffffff"));
    let n_prop_inner_max_path = path_partial(n_prop_inner_max_data, Some("#ffffff"), Some(0.5));
    let n_prop_outer_max_path = path_partial(n_prop_outer_max_data, Some("#ffffff"), Some(0.5));

    let longest_arc_path = if show_longest {
        path_filled(longest_arc_data, Some("#e31a1c"))
    } else {
        Path::new()
    };
    let n50_arc_path = path_filled(n50_arc_data, Some("#ff7f00"));
    let n90_arc_path = path_filled(n90_arc_data, Some("#fdbf6f"));
    let n50_arc_outline_path = path_open(n50_arc_outline_data, Some("#ff7f00"), None);
    let longest_arc_outline_path = path_open(longest_arc_outline_data, Some("#e31a1c"), None);
    let inner = path_axis_major(axis_arc_data, None);
    let outer = path_axis_major(outer_axis_arc_data, None);

    let inner_axis = Line::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 3)
        .set("x1", 0.0)
        .set("y1", 0.0)
        .set("x2", 0.0)
        .set("y2", -radius);

    let mut major_tick_group = Group::new();
    for tick in major_ticks {
        major_tick_group = major_tick_group
            .add(tick.path)
            .add(tick.label)
            .add(tick.outer_label)
    }
    let mut minor_tick_group = Group::new();
    for tick in minor_ticks {
        minor_tick_group = minor_tick_group.add(tick.path)
    }

    let mut major_length_tick_group = Group::new();
    let mut major_length_gridline_group = Group::new();

    for tick in major_length_ticks {
        major_length_tick_group = major_length_tick_group.add(tick.path).add(tick.label);
        let arc_data = arc_path(
            -1.0 * tick.position,
            None,
            -PI / 2.0,
            max_radians - PI / 2.0,
            options.segments,
        );
        major_length_gridline_group =
            major_length_gridline_group.add(path_gridline_minor(arc_data, Some("#ffffff")));
    }

    let mut major_count_gridline_group = Group::new();
    let mut i = 10;
    while i <= snail_stats.scaffold_count() {
        let arc_data = arc_path(
            log_scale(i, &scaf_count_domain, &scaf_count_range),
            None,
            -PI / 2.0,
            max_radians - PI / 2.0,
            options.segments,
        );
        major_count_gridline_group =
            major_count_gridline_group.add(path_gridline_major(arc_data, Some("#ffffff")));
        i *= 10;
    }

    let mut minor_length_tick_group = Group::new();
    for tick in minor_length_ticks {
        minor_length_tick_group = minor_length_tick_group.add(tick.path)
    }

    let scaf_stats_legend = scaffold_stats_legend(&snail_stats, &options)
        .set("transform", format!("translate({},{})", 5, 25));

    let comp_stats_legend = composition_stats_legend(&snail_stats, &options)
        .set("transform", format!("translate({},{})", 835, 900));

    let scale_legend = scale_stats_legend(&snail_stats, &options)
        .set("transform", format!("translate({},{})", 5, 900));

    let dataset_legend = dataset_name_legend(&snail_stats, &options)
        .set("transform", format!("translate({},{})", 5, 990));

    let busc_stats_legend = busco_stats_legend(&snail_stats, &options)
        .set("transform", format!("translate({},{})", 630, 25));

    let busco_group = busco_plot(snail_stats).set("transform", "translate(910, 170)");

    let group = Group::new()
        .set("transform", "translate(500, 525)")
        .add(scaf_count_path)
        .add(major_count_gridline_group)
        .add(scaf_length_path)
        .add(gc_prop_path)
        .add(at_prop_path)
        .add(n_prop_inner_path)
        .add(n_prop_outer_path)
        .add(n_prop_inner_max_path)
        .add(n_prop_outer_max_path)
        .add(gc_prop_max_path)
        .add(gc_prop_min_path)
        .add(longest_arc_path)
        .add(n50_arc_path)
        .add(n90_arc_path)
        .add(major_length_gridline_group)
        .add(n50_arc_outline_path)
        .add(longest_arc_outline_path)
        .add(minor_tick_group)
        .add(major_tick_group)
        .add(minor_length_tick_group)
        .add(major_length_tick_group)
        .add(inner_axis)
        // .add(outer_axis)
        .add(inner)
        .add(outer);

    let document = Document::new()
        .set("viewBox", (0, 0, 1000, 1000))
        .add(group)
        .add(busco_group)
        .add(scaf_stats_legend)
        .add(comp_stats_legend)
        .add(busc_stats_legend)
        .add(scale_legend)
        .add(dataset_legend);

    svg::save(options.output.as_str(), &document).unwrap();
}

fn busco_plot(snail_stats: &SnailStats) -> Group {
    let domain = [0.0, snail_stats.busco_total() as f64];
    let range = [-PI / 2.0, PI * 1.5];
    let inner_radius = 20.0;
    let outer_radius = 60.0;
    let comp_arc_data = arc_path(
        outer_radius,
        Some(inner_radius),
        -PI / 2.0,
        linear_scale_float(snail_stats.busco_complete() as f64, &domain, &range),
        1000,
    );
    let comp_arc_path = path_filled(comp_arc_data, Some("#33a02c"));
    let frag_arc_data = arc_path(
        outer_radius,
        Some(inner_radius),
        linear_scale_float(snail_stats.busco_complete() as f64, &domain, &range),
        linear_scale_float(
            (snail_stats.busco_fragmented() + snail_stats.busco_complete()) as f64,
            &domain,
            &range,
        ),
        1000,
    );
    let frag_arc_path = path_filled(frag_arc_data, Some("#a3e27f"));
    let dup_arc_data = arc_path(
        outer_radius,
        Some(inner_radius),
        -PI / 2.0,
        linear_scale_float(snail_stats.busco_duplicated() as f64, &domain, &range),
        1000,
    );
    let dup_arc_path = path_filled(dup_arc_data, Some("#20641b"));
    let major_ticks = set_axis_ticks_circular(
        1000,
        10,
        TickStatus::Major,
        2.0 * PI,
        outer_radius,
        outer_radius + 20.0,
        100,
        TickOptions {
            font_size: 14.0,
            ..Default::default()
        },
    );
    let mut major_tick_group = Group::new();
    for tick in major_ticks {
        major_tick_group = major_tick_group.add(tick.path).add(tick.label)
    }
    let minor_ticks = set_axis_ticks_circular(
        1000,
        50,
        TickStatus::Minor,
        2.0 * PI,
        outer_radius,
        outer_radius + 20.0,
        100,
        TickOptions {
            ..Default::default()
        },
    );
    let mut minor_tick_group = Group::new();
    for tick in minor_ticks {
        minor_tick_group = minor_tick_group.add(tick.path)
    }

    let cirular_axis_data = arc_path(outer_radius, None, -PI / 2.0, PI * 1.5, 1000);
    let circular_axis_path = path_axis_minor(cirular_axis_data, None);

    let radial_axis = Line::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 1)
        .set("x1", 0.0)
        .set("y1", 0.0)
        .set("x2", 0.0)
        .set("y2", -outer_radius);

    let busco_group = Group::new()
        .add(comp_arc_path)
        .add(frag_arc_path)
        .add(dup_arc_path)
        .add(minor_tick_group)
        .add(major_tick_group)
        .add(radial_axis)
        .add(circular_axis_path);

    busco_group
}

fn arc_path(
    radius: f64,
    inner_radius: Option<f64>,
    min_radians: f64,
    max_radians: f64,
    resolution: usize,
) -> Data {
    let mut path_data = Data::new();

    let step = (max_radians - min_radians) / resolution as f64;
    let length = resolution;

    let first_polar_coord = Vector2::new(radius, min_radians);
    let first_cartesian_coord = polar2cartesian(&first_polar_coord);

    match inner_radius {
        None => path_data = path_data.move_to((first_cartesian_coord[0], first_cartesian_coord[1])),
        Some(rad) => {
            if rad == 0.0 {
                let polar_coord = Vector2::new(0.0, 0.0);
                let cartesian_coord = polar2cartesian(&polar_coord);
                path_data = path_data
                    .move_to((cartesian_coord[0], cartesian_coord[1]))
                    .line_to((first_cartesian_coord[0], first_cartesian_coord[1]));
            } else {
                let mut angle = max_radians;
                for i in (0..length + 1).rev() {
                    let polar_coord = Vector2::new(rad, angle);
                    let cartesian_coord = polar2cartesian(&polar_coord);
                    if i == length {
                        path_data = path_data.move_to((cartesian_coord[0], cartesian_coord[1]))
                    } else {
                        path_data = path_data.line_to((cartesian_coord[0], cartesian_coord[1]))
                    }
                    angle -= step;
                }
                path_data = path_data.line_to((first_cartesian_coord[0], first_cartesian_coord[1]))
            }
        }
    };

    let mut angle = min_radians;
    for _ in 0..(length + 1) {
        let polar_coord = Vector2::new(radius, angle);
        let cartesian_coord = polar2cartesian(&polar_coord);
        path_data = path_data.line_to((cartesian_coord[0], cartesian_coord[1]));
        angle += step;
    }

    match inner_radius {
        None => (),
        Some(_) => path_data = path_data.close(),
    };

    path_data
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
        let mut polar_coord_start;
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

fn polar_to_path_bounded(
    polar_coords: &Vec<Vec<f64>>,
    polar_bound_coords: &Vec<Vec<f64>>,
    bin_count: usize,
    max_radians: f64,
) -> Data {
    let mut path_data = Data::new();

    let mut length = polar_coords.len();
    if length == 0 {
        length = bin_count;
    }

    // for i in (0..length).rev() {
    //     let angle = if polar_coords.len() > 0 {
    //         polar_coords[i][1]
    //     } else {
    //         max_radians * (i + 1) as f64 / length as f64 - PI / 2.0
    //     };
    //     let polar_coord = Vector2::new(radius, angle);
    //     let cartesian_coord = polar2cartesian(&polar_coord);
    //     if i == length - 1 {
    //         path_data = path_data.move_to((cartesian_coord[0], cartesian_coord[1]))
    //     } else {
    //         path_data = path_data.line_to((cartesian_coord[0], cartesian_coord[1]))
    //     }
    //     if i == 0 {
    //         let final_polar_coord = Vector2::new(radius, -PI / 2.0);
    //         let final_cartesian_coord = polar2cartesian(&final_polar_coord);
    //         path_data = path_data.line_to((final_cartesian_coord[0], final_cartesian_coord[1]))
    //     }
    // }

    for i in (0..polar_bound_coords.len()).rev() {
        let polar_coord_end;
        let polar_coord_start;
        if i < polar_bound_coords.len() - 1 {
            polar_coord_start =
                Vector2::new(polar_bound_coords[i + 1][0], polar_bound_coords[i + 1][1]);
            polar_coord_end = Vector2::new(polar_bound_coords[i + 1][0], polar_bound_coords[i][1]);
        } else {
            polar_coord_start = Vector2::new(polar_bound_coords[i][0], max_radians - PI / 2.0);
            polar_coord_end = Vector2::new(polar_bound_coords[i][0], polar_bound_coords[i][1]);
        };

        let cartesian_start = polar2cartesian(&polar_coord_start);
        let cartesian_end = polar2cartesian(&polar_coord_end);
        if i == length - 1 {
            path_data = path_data
                .move_to((cartesian_start[0], cartesian_start[1]))
                .line_to((cartesian_end[0], cartesian_end[1]));
        } else {
            path_data = path_data
                .line_to((cartesian_start[0], cartesian_start[1]))
                .line_to((cartesian_end[0], cartesian_end[1]));
        }
        if i == 0 {
            let final_polar_coord = Vector2::new(polar_bound_coords[i][0], -PI / 2.0);
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
    }
    if polar_coords.len() > 0 {
        path_data = path_data.close();
    }
    path_data
}
