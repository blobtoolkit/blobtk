use std::cmp::Ordering;
use std::collections::HashSet;
use std::f64::consts::PI;

use serde;
use serde::{Deserialize, Serialize};
use svg::node::element::{Group, Line, Path, Rectangle, Text};
use svg::Document;
use titlecase::titlecase;

use crate::blobdir::{self, BuscoGene};

use super::axis::{TickOptions, TickStatus};
use super::component::{
    arc_path, legend, path_axis_major, path_axis_minor, path_gridline_major, path_gridline_minor,
    polar_to_path, polar_to_path_bounded, set_axis_ticks, set_axis_ticks_circular, LegendEntry,
    LegendShape,
};
use super::style::{path_filled, path_open, path_partial};
use crate::cli;
use crate::utils::{
    self, compact_float, format_si, linear_scale, linear_scale_float, log_scale, sqrt_scale,
};

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
        } else {
            if busco_list.contains(&busco_id) {
                busco_dup.insert(busco_id.clone());
            }
            busco_list.insert(busco_id);
        }
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
    let order = utils::indexed_sort(&length_values);
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
    entries.push(LegendEntry {
        title: format!("Log10 {} count (total {})", record, scaffold_count),
        color: "#dddddd".to_string(),
        ..Default::default()
    });
    entries.push(LegendEntry {
        title: format!("{} length (total {})", titlecase(record), scaffold_length),
        color: "#999999".to_string(),
        ..Default::default()
    });
    entries.push(LegendEntry {
        title: format!("Longest {} ({})", record, longest_scaffold),
        color: "#e31a1c".to_string(),
        ..Default::default()
    });
    entries.push(LegendEntry {
        title: format!("N50 length ({})", n50_length),
        color: "#ff7f00".to_string(),
        ..Default::default()
    });
    entries.push(LegendEntry {
        title: format!("N90 length ({})", n90_length),
        color: "#fdbf6f".to_string(),
        ..Default::default()
    });

    let title = format!("{} statistics", titlecase(record));
    legend(title, entries, None, 1)
}

pub fn composition_stats_legend(snail_stats: &SnailStats, _: &cli::PlotOptions) -> Group {
    let mut entries = vec![];
    let gc_prop = format_si(&(snail_stats.gc_proportion as f64 * 100.0), 3);
    let at_prop = format_si(&(snail_stats.at_proportion as f64 * 100.0), 3);
    let n_prop = format_si(&(snail_stats.n_proportion as f64 * 100.0), 3);
    entries.push(LegendEntry {
        title: format!("GC ({}%)", gc_prop),
        color: "#1f78b4".to_string(),
        ..Default::default()
    });
    entries.push(LegendEntry {
        title: format!("AT ({}%)", at_prop),
        color: "#a6cee3".to_string(),
        ..Default::default()
    });
    entries.push(LegendEntry {
        title: format!("N ({}%)", n_prop),
        color: "#ffffff".to_string(),
        ..Default::default()
    });

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
    entries.push(LegendEntry {
        title: format!("{}", circ_prop),
        color: "#ffffff".to_string(),
        shape: LegendShape::Circumference,
        ..Default::default()
    });
    entries.push(LegendEntry {
        title: format!("{}", rad_prop),
        color: "#ffffff".to_string(),
        shape: LegendShape::Radius,
        ..Default::default()
    });

    let title = "Scale".to_string();
    legend(title, entries, None, 1)
}

pub fn dataset_name_legend(snail_stats: &SnailStats, _: &cli::PlotOptions) -> Group {
    let entries = vec![];

    let title = format!("Dataset: {}", snail_stats.id);
    legend(title, entries, None, 1)
}

pub fn busco_stats_legend(snail_stats: &SnailStats, _: &cli::PlotOptions) -> Group {
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
    entries.push(LegendEntry {
        title: format!("Comp. ({}%)", comp_prop),
        color: "#33a02c".to_string(),
        ..Default::default()
    });
    entries.push(LegendEntry {
        title: format!("Dupl. ({}%)", dup_prop),
        color: "#20641b".to_string(),
        ..Default::default()
    });
    entries.push(LegendEntry {
        title: format!("Frag. ({}%)", frag_prop),
        color: "#a3e27f".to_string(),
        ..Default::default()
    });
    entries.push(LegendEntry {
        title: format!("Missing ({}%)", missing_prop),
        color: "#ffffff".to_string(),
        ..Default::default()
    });

    let title = "BUSCO".to_string();
    legend(title, entries, Some(subtitle), 2)
}

pub fn svg(snail_stats: &SnailStats, options: &cli::PlotOptions) -> Document {
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
    let min_scaffold = snail_stats.binned_scaffold_lengths()[bin_count - 1];
    let mut magnitude = (min_scaffold.clone() as f64).log10() as u32;
    if magnitude > 1 {
        magnitude -= 1;
    }
    let min_value = 10u32.pow(magnitude) as usize;

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
            label_ticks: true,
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
            label_ticks: true,
            ..Default::default()
        },
    );
    let major_length_ticks = set_axis_ticks(
        &(max_scaffold as f64),
        &(min_value as f64),
        &TickStatus::Major,
        &radius,
        &"scaleSqrt".to_string(),
    );
    let minor_length_ticks = set_axis_ticks(
        &(max_scaffold as f64),
        &(min_value as f64),
        &TickStatus::Minor,
        &radius,
        &"scaleSqrt".to_string(),
    );
    let scaled_n50 = sqrt_scale(
        snail_stats.binned_scaffold_lengths()[n50_index],
        &[min_value, max_scaffold],
        &[radius, 0.0],
    );
    let scaled_n90 = sqrt_scale(
        snail_stats.binned_scaffold_lengths()[n90_index],
        &[min_value, max_scaffold],
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
    let scaf_count_domain = [1, 10000000000];
    let scaf_count_range = [0.0, radius];
    for i in 0..bin_count {
        // angle
        let angle = linear_scale(i + 1, &[0, bin_count], &[-PI / 2.0, max_radians - PI / 2.0]);

        // scaffold lengths
        let scaf_length_polar: Vec<f64> = vec![
            sqrt_scale(
                snail_stats.binned_scaffold_lengths()[i],
                &[min_value, max_scaffold],
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

    for (i, tick) in major_length_ticks.iter().enumerate() {
        let tick = tick.clone();
        let label = if i < major_length_ticks.len() - 3 {
            Text::new()
        } else {
            tick.label
        };
        major_length_tick_group = major_length_tick_group.add(tick.path).add(label);
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
        .add(
            Rectangle::new()
                .set("fill", "#ffffff")
                .set("stroke", "none")
                .set("width", 1000)
                .set("height", 1000),
        )
        .add(scaf_stats_legend)
        .add(comp_stats_legend)
        .add(busc_stats_legend)
        .add(scale_legend)
        .add(dataset_legend)
        .add(group)
        .add(busco_group);

    // svg::save(options.output.as_str(), &document).unwrap();
    // let mut target = Vec::new();
    // let svg_data = svg::write(target, &document).unwrap();
    document
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
