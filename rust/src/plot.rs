//!
//! Invoked by calling:
//! `blobtk plot <args>`

// use std::collections::HashMap;

use std::borrow::BorrowMut;
use std::collections::HashMap;

use crate::blobdir;
use crate::cli;
// use crate::io;

pub use cli::PlotOptions;
use colorous;
use svg::Document;
use usvg::{fontdb, TreeParsing, TreeTextToPath};

/// Blob plot functions.
pub mod blob;

/// Chart components.
pub mod component;

/// Snail plot functions.
pub mod snail;

/// SVG styling functions.
pub mod style;

pub fn save_svg(document: &Document, options: &PlotOptions) {
    svg::save(options.output.as_str(), document).unwrap();
}

pub fn save_png(document: &Document, _: &PlotOptions) {
    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();
    let mut buf = Vec::new();
    svg::write(&mut buf, document).unwrap();
    let opt = usvg::Options::default();
    let mut tree = usvg::Tree::from_data(&buf.as_slice(), &opt).unwrap();
    tree.convert_text(&fontdb);

    let pixmap_size = tree.size.to_screen_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(
        &tree,
        resvg::FitTo::Original,
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .unwrap();
    pixmap.save_png("test.png").unwrap();
}

pub fn plot_snail(meta: &blobdir::Meta, options: &cli::PlotOptions) {
    // let busco_list = meta.busco_list.clone().unwrap();
    let busco_field = meta.busco_list.clone().unwrap()[0].clone();
    let busco_values = blobdir::parse_field_busco(busco_field.0, &options.blobdir).unwrap();
    let busco_total = busco_field.1;
    let busco_lineage = busco_field.2;
    let gc_values = blobdir::parse_field_float("gc".to_string(), &options.blobdir).unwrap();
    let length_values = blobdir::parse_field_int("length".to_string(), &options.blobdir).unwrap();
    let n_values = blobdir::parse_field_float("n".to_string(), &options.blobdir);
    let ncount_values = blobdir::parse_field_int("ncount".to_string(), &options.blobdir).unwrap();
    let id = meta.id.clone();
    let record_type = meta.record_type.clone();

    let filters = blobdir::parse_filters(&options.filter);
    let wanted_indices = blobdir::set_filters(filters, &meta, &options.blobdir);

    let gc_filtered = blobdir::apply_filter_float(&gc_values, &wanted_indices);
    let n_filtered = match n_values {
        None => None,
        Some(values) => Some(blobdir::apply_filter_float(&values, &wanted_indices)),
    };
    let length_filtered = blobdir::apply_filter_int(&length_values, &wanted_indices);
    let ncount_filtered = blobdir::apply_filter_int(&ncount_values, &wanted_indices);
    let busco_filtered = blobdir::apply_filter_busco(&busco_values, &wanted_indices);

    let snail_stats = snail::snail_stats(
        &length_filtered,
        &gc_filtered,
        &n_filtered,
        &ncount_filtered,
        &busco_filtered,
        busco_total,
        busco_lineage,
        id,
        record_type,
        &options,
    );
    let document: Document = snail::svg(&snail_stats, &options);

    save_svg(&document, &options);

    save_png(&document, &options);

    // let cats = blobdir::parse_field_cat("buscogenes_family".to_string(), &options);
    // let identifiers = blobdir::parse_field_string("identifiers".to_string(), &options);
}

pub fn color_to_hex(color: colorous::Color) -> String {
    format!("#{:x}{:x}{:x}", color.r, color.g, color.b)
}

pub fn default_palette(count: usize) -> Vec<String> {
    let gradient = colorous::PAIRED;
    let mut list = vec![];
    for i in 0..count {
        let mut j = if i % 2 == 1 { i - 1 } else { i + 1 };
        j = j % 12;
        list.push(color_to_hex(gradient[j]));
    }
    list
}

pub fn set_palette(
    name: &Option<cli::Palette>,
    colors: &Option<Vec<String>>,
    count: usize,
) -> Vec<String> {
    let mut color_list = match name {
        Some(cli::Palette::Default) => default_palette(count),
        Some(cli::Palette::Paired) => {
            let gradient = colorous::PAIRED;
            let mut list = vec![];
            for i in 0..count {
                let j = i % 12;
                list.push(color_to_hex(gradient[j]));
            }
            list
        }
        Some(cli::Palette::Viridis) => {
            let gradient = colorous::VIRIDIS;
            (0..count)
                .map(|i| color_to_hex(gradient.eval_rational(i, count)))
                .collect()
        }
        None => default_palette(count),
    };
    if colors.is_some() {
        for color in colors.clone().unwrap() {
            let (index, hex) = color.split_once("=").unwrap();
            let i: usize = index.parse().unwrap();
            let mut hexcode = hex.to_string();
            if i <= count {
                hexcode = hexcode.replace("hex", "#");
                if !hexcode.starts_with("#") {
                    hexcode = format!("#{}", hexcode);
                }
                color_list[i] = hexcode;
            }
        }
    }
    color_list
}

#[derive(Clone, Debug)]
pub struct Category {
    label: String,
    members: Vec<String>,
    indices: Vec<usize>,
    color: String,
}

pub fn set_cats(
    values: &Vec<(String, usize)>,
    order: &Option<String>,
    count: &usize,
    palette: &Vec<String>,
) -> Vec<Category> {
    let mut indices = HashMap::new();
    let mut label_list = vec![];
    for (i, entry) in values.iter().enumerate() {
        label_list.push(entry.clone().0);
        if !indices.contains_key(&entry.0) {
            indices.insert(entry.clone().0, vec![i]);
        } else {
            let list = indices.get_mut(&entry.0).unwrap();
            list.push(i);
        }
    }
    let frequencies = values
        .iter()
        .map(|x| x.clone().0)
        .fold(HashMap::new(), |mut map, val| {
            map.entry(val).and_modify(|frq| *frq += 1).or_insert(1);
            map
        });
    let mut sorted_cats: Vec<_> = frequencies.clone().into_iter().collect();
    sorted_cats.sort_by(|x, y| y.1.cmp(&x.1));

    let mut cat_order = vec![];
    let mut index = 0;
    if order.is_some() {
        // TODO: prevent duplication when adding remaining cats
        for entry in order.clone().unwrap().split(",") {
            if frequencies.contains_key(entry) {
                cat_order.push(Category {
                    label: entry.to_string(),
                    members: vec![],
                    indices: vec![],
                    color: palette[index].clone(),
                });
                index += 1;
            }
        }
    }
    for (label, _) in &sorted_cats {
        if index < count - 1 || index == count - 1 && *count == sorted_cats.len() {
            cat_order.push(Category {
                label: label.clone(),
                members: vec![label.clone()],
                indices: indices[label].clone(),
                color: palette[index].clone(),
            });
            index += 1
        } else if cat_order.len() < *count {
            cat_order.push(Category {
                label: "other".to_string(),
                members: vec![label.clone()],
                indices: indices[label].clone(),
                color: palette[count - 1].clone(),
            });
        } else {
            let other_cat = cat_order[count - 1].borrow_mut();
            other_cat.members.push(label.clone());
            other_cat.indices.append(indices.get_mut(label).unwrap());
        }
    }
    cat_order
}

pub fn plot_blob(meta: &blobdir::Meta, options: &cli::PlotOptions) {
    // let busco_list = meta.busco_list.clone().unwrap();
    let mut plot_meta: HashMap<String, String> = HashMap::new();
    if options.x_field.is_some() {
        plot_meta.insert("x".to_string(), options.x_field.clone().unwrap());
    } else {
        plot_meta.insert("x".to_string(), meta.plot.x.clone().unwrap());
    }
    if options.y_field.is_some() {
        plot_meta.insert("y".to_string(), options.y_field.clone().unwrap());
    } else {
        plot_meta.insert("y".to_string(), meta.plot.y.clone().unwrap());
    }
    if options.z_field.is_some() {
        plot_meta.insert("z".to_string(), options.z_field.clone().unwrap());
    } else {
        plot_meta.insert("z".to_string(), meta.plot.z.clone().unwrap());
    }
    if options.cat_field.is_some() {
        plot_meta.insert("cat".to_string(), options.cat_field.clone().unwrap());
    } else {
        plot_meta.insert("cat".to_string(), meta.plot.cat.clone().unwrap());
    }
    // TODO: handle empty values

    let (plot_values, cat_values) = blobdir::get_plot_values(&meta, &options.blobdir, &plot_meta);

    let palette = set_palette(&options.palette, &options.color, options.cat_count);

    let cat_order = set_cats(
        &cat_values,
        &options.cat_order,
        &options.cat_count,
        &palette,
    );
    dbg!(cat_order);
    // let id = meta.id.clone();
    // let record_type = meta.record_type.clone();

    let filters = blobdir::parse_filters(&options.filter);
    let wanted_indices = blobdir::set_filters(filters, &meta, &options.blobdir);
    let x_filtered = blobdir::apply_filter_float(&plot_values["x"], &wanted_indices);
    let y_filtered = blobdir::apply_filter_float(&plot_values["y"], &wanted_indices);
    let z_filtered = blobdir::apply_filter_float(&plot_values["z"], &wanted_indices);
    let cat_filtered = blobdir::apply_filter_cat(&cat_values, &wanted_indices);
}

/// Execute the `plot` subcommand from `blobtk`.
pub fn plot(options: &cli::PlotOptions) -> Result<(), Box<dyn std::error::Error>> {
    let meta = blobdir::parse_blobdir(&options.blobdir);
    let view = &options.view;
    match view {
        Some(cli::View::Blob) => plot_blob(&meta, &options),
        Some(cli::View::Snail) => plot_snail(&meta, &options),
        _ => (),
    }
    Ok(())
}
