use svg::node::element::path::Data;
use svg::node::element::Path;

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
