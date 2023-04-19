//!
//! Invoked by calling:
//! `blobtk plot <args>`

// use std::collections::HashMap;

use crate::blobdir;
use crate::cli;
use crate::snail;
// use crate::io;

pub use cli::PlotOptions;
use svg::Document;
use usvg::{fontdb, TreeParsing, TreeTextToPath};

pub fn save_svg(document: &Document, options: &PlotOptions) {
    svg::save(options.output.as_str(), document).unwrap();
}

pub fn save_png(document: &Document, _: &PlotOptions) {
    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();
    let mut buf = Vec::new();
    svg::write(&mut buf, document).unwrap();
    // let output = std::str::from_utf8(buf.as_slice()).unwrap().to_string();
    // dbg!(output);
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
    let busco_values = blobdir::parse_field_busco(busco_field.0, &options).unwrap();
    let busco_total = busco_field.1;
    let busco_lineage = busco_field.2;
    let gc_values = blobdir::parse_field_float("gc".to_string(), &options).unwrap();
    let length_values = blobdir::parse_field_int("length".to_string(), &options).unwrap();
    let n_values = blobdir::parse_field_float("n".to_string(), &options);
    let ncount_values = blobdir::parse_field_int("ncount".to_string(), &options).unwrap();
    let id = meta.id.clone();
    let record_type = meta.record_type.clone();

    let snail_stats = snail::snail_stats(
        &length_values,
        &gc_values,
        &n_values,
        &ncount_values,
        &busco_values,
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

/// Execute the `plot` subcommand from `blobtk`.
pub fn plot(options: &cli::PlotOptions) -> Result<(), Box<dyn std::error::Error>> {
    let meta = blobdir::parse_blobdir(&options);
    let view = &options.view;
    match view {
        Some(cli::View::Snail) => plot_snail(&meta, &options),
        _ => (),
    }
    Ok(())
}
