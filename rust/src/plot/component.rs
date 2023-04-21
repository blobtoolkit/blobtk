use std::f64::consts::PI;

use coord_transforms::d2::polar2cartesian;
use coord_transforms::prelude::*;
use num_integer::div_rem;
use svg::node::element::path::Data;
use svg::node::element::{Circle, Group, Line, Path, Rectangle, Text};
use svg::node::Text as nodeText;

use crate::utils::{format_si, linear_scale, linear_scale_float, scale_float};

use super::scatter::ScatterAxis;

#[derive(Clone, Debug)]
pub struct RadialTick {
    pub index: usize,
    pub offset: f64,
    pub angle: f64,
    pub label: Text,
    pub path: Path,
    pub outer_label: Text,
    pub midpoint: (f64, f64),
    pub status: TickStatus,
}

#[derive(Clone, Debug)]
pub struct Tick {
    pub label: Text,
    pub path: Path,
    pub position: f64,
    pub status: TickStatus,
}

#[derive(Clone, Debug)]
pub struct TickOptions {
    pub font_size: f64,
    pub font_color: String,
    pub show_secondary_tick: bool,
    pub status: TickStatus,
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

#[derive(Clone, Debug)]
pub enum TickStatus {
    Major,
    Minor,
}

#[derive(Clone, Debug)]
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

pub fn set_tick(
    value: f64,
    label: String,
    domain: &[f64; 2],
    range: &[f64; 2],
    status: &TickStatus,
    scale: &String,
) -> Tick {
    let offset = scale_float(value, &domain, &range, &scale, None);
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

pub fn set_tick_horiz(
    value: f64,
    label: String,
    domain: &[f64; 2],
    range: &[f64; 2],
    status: &TickStatus,
    scale: &String,
) -> Tick {
    let offset = scale_float(value, &domain, &range, &scale, None);
    let path = match status {
        TickStatus::Major => {
            path_axis_major(Data::new().move_to((offset, 10)).line_to((offset, 0)), None)
        }
        TickStatus::Minor => {
            path_axis_minor(Data::new().move_to((offset, 5)).line_to((offset, 0)), None)
        }
    };
    let text = match status {
        TickStatus::Major => Text::new()
            .set("font-family", "Roboto, Open sans, sans-serif")
            .set("font-size", "20")
            .set("text-anchor", "middle")
            .set("dominant-baseline", "hanging")
            .set("stroke", "none")
            .set("fill", "black")
            .set("transform", format!("translate({:?}, {:?})", offset, 15,))
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
    scale: &String,
) -> Vec<Tick> {
    let range = [-dimension.clone(), 0.0];
    let domain = [min_value.clone(), max_value.clone()];

    if scale == &"scaleLog".to_string() {}
    let mut power: i32 = 0;
    let mut min_val = min_value.clone().abs();

    if min_val > 1.0 {
        while min_val > 1.0 {
            power += 1;
            min_val /= 10.0;
        }
    } else {
        while min_val < 1.0 {
            power -= 1;
            min_val *= 10.0;
        }
    }

    let mut ticks: Vec<Tick> = vec![];
    match status {
        TickStatus::Major => {
            let mut i = 10u32.pow(power.abs() as u32) as f64;
            if power < 0 {
                i = 1.0 / i;
            }
            if min_value.clone() < 0.0 {
                i = -i
            }
            while i <= max_value.clone() {
                let label = if i > min_value.clone() {
                    format_si(&i, 3)
                } else {
                    String::new()
                };
                if false {
                    ticks.push(set_tick_horiz(i, label, &domain, &range, &status, &scale));
                } else {
                    ticks.push(set_tick(i, label, &domain, &range, &status, &scale));
                }
                i = i * 10.0;
            }
        }
        TickStatus::Minor => {
            let mut i = 10u32.pow((power.abs() - 1) as u32) as f64;
            if power < 0 {
                i = 1.0 / i;
            }
            if min_value.clone() < 0.0 {
                i = -i
            }
            while i <= max_value.clone() {
                let mut j = i * 2.0;
                while j < i * 10.0 && j <= max_value.clone() {
                    if &(j as f64) >= min_value {
                        ticks.push(set_tick(j, String::new(), &domain, &range, &status, &scale));
                    }
                    j = j + i;
                }
                ticks.push(set_tick(i, String::new(), &domain, &range, &status, &scale));
                i = i * 10.0;
            }
        }
    }
    ticks
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

pub fn arc_path(
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

pub fn polar_to_path(
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
        let polar_coord_start;
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

pub fn polar_to_path_bounded(
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
        let polar_coord_start;
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

pub fn x_axis(scatter_axis: &ScatterAxis) -> Group {
    let major_ticks = set_axis_ticks(
        &scatter_axis.domain[1],
        &scatter_axis.domain[0],
        &TickStatus::Major,
        &(scatter_axis.range[1] - scatter_axis.range[0]),
        &scatter_axis.scale,
    );
    let mut major_tick_group = Group::new();
    for tick in major_ticks {
        major_tick_group = major_tick_group.add(tick.path).add(tick.label);
    }

    let minor_ticks = set_axis_ticks(
        &scatter_axis.domain[1],
        &scatter_axis.domain[0],
        &TickStatus::Minor,
        &(scatter_axis.range[1] - scatter_axis.range[0]),
        &scatter_axis.scale,
    );
    let mut minor_tick_group = Group::new();
    for tick in minor_ticks {
        minor_tick_group = minor_tick_group.add(tick.path);
    }

    let axis = Line::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 3)
        .set("x1", scatter_axis.range[0])
        .set("y1", 0.0)
        .set("x2", scatter_axis.range[1])
        .set("y2", 0.0);

    Group::new()
        .add(minor_tick_group)
        .add(major_tick_group)
        .add(axis)
}

pub fn y_axis(scatter_axis: &ScatterAxis) -> Group {
    let major_ticks = set_axis_ticks(
        &scatter_axis.domain[1],
        &scatter_axis.domain[0],
        &TickStatus::Major,
        &(scatter_axis.range[1] - scatter_axis.range[0]),
        &scatter_axis.scale,
    );
    let mut major_tick_group = Group::new();
    for tick in major_ticks {
        major_tick_group = major_tick_group.add(tick.path).add(tick.label);
    }

    let minor_ticks = set_axis_ticks(
        &scatter_axis.domain[1],
        &scatter_axis.domain[0],
        &TickStatus::Minor,
        &(scatter_axis.range[1] - scatter_axis.range[0]),
        &scatter_axis.scale,
    );
    let mut minor_tick_group = Group::new();
    for tick in minor_ticks {
        minor_tick_group = minor_tick_group.add(tick.path);
    }

    let axis = Line::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 3)
        .set("x1", 0.0)
        .set("y1", scatter_axis.range[0])
        .set("x2", 0.0)
        .set("y2", scatter_axis.range[1]);

    Group::new()
        .add(minor_tick_group)
        .add(major_tick_group)
        .add(axis)
}
