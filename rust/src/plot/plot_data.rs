use svg::node::element::path::Data;

use super::axis::{AxisName, AxisOptions};
use super::category::Category;

#[derive(Clone, Debug)]
pub struct ScatterPoint {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub label: Option<String>,
    pub color: Option<String>,
}

impl Default for ScatterPoint {
    fn default() -> ScatterPoint {
        ScatterPoint {
            x: 0.0,
            y: 0.0,
            z: 5.0,
            label: None,
            color: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScatterData {
    pub points: Vec<ScatterPoint>,
    pub x: AxisOptions,
    pub y: AxisOptions,
    pub z: AxisOptions,
    pub categories: Vec<Category>,
}

#[derive(Clone, Debug)]
pub struct Bin {
    pub height: f64,
    pub width: f64,
    pub value: f64,
}

#[derive(Clone, Debug)]
pub struct HistogramData {
    pub bins: Vec<Bin>,
    pub max_bin: f64,
    pub axis: AxisName,
    pub category: Option<Category>,
}

impl Default for HistogramData {
    fn default() -> HistogramData {
        HistogramData {
            bins: vec![],
            max_bin: 0.0,
            axis: AxisName::X,
            category: None,
        }
    }
}

impl HistogramData {
    pub fn to_path_data(self, filled: bool) -> Data {
        let mut path_data;
        let mut offset = 0.0;
        if filled {
            path_data = Data::new().move_to((0.0, 0.0));
            for (i, bin) in self.bins.iter().enumerate() {
                path_data = path_data
                    .line_to((offset, -bin.height))
                    .line_to((offset + bin.width, -bin.height));
                dbg!(&offset, &bin.height);
                offset += bin.width;
            }
            path_data = path_data.line_to((offset, 0.0)).close();
        } else {
            path_data = Data::new().move_to((0.0, 0.0));
            for bin in &self.bins {
                if path_data.is_empty() {
                    path_data = path_data
                        .move_to((offset, -bin.height))
                        .line_to((offset + bin.width, -bin.height));
                } else {
                    path_data = path_data
                        .line_to((offset, -bin.height))
                        .line_to((offset + bin.width, -bin.height));
                }
                offset += bin.width;
            }
        }
        path_data
    }
}
