use svg::node::element::path::Data;

use super::axis::{AxisName, AxisOptions, Position};
use super::category::Category;

#[derive(Clone, Debug)]
pub struct ScatterPoint {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub label: Option<String>,
    pub color: Option<String>,
    pub cat_index: usize,
    pub data_index: usize,
}

impl Default for ScatterPoint {
    fn default() -> ScatterPoint {
        ScatterPoint {
            x: 0.0,
            y: 0.0,
            z: 5.0,
            label: None,
            color: None,
            cat_index: 0,
            data_index: 0,
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
    pub width: f64,
    pub axis: AxisName,
    pub category: Option<Category>,
}

impl Default for HistogramData {
    fn default() -> HistogramData {
        HistogramData {
            bins: vec![],
            max_bin: 0.0,
            width: 100.0,
            axis: AxisName::X,
            category: None,
        }
    }
}

impl HistogramData {
    pub fn to_path_data(self, position: Position, _filled: bool) -> Data {
        let shift = self.max_bin;
        let (mut offset, mut path_data) = match position {
            Position::TOP | Position::BOTTOM => (0.0, Data::new().move_to((0.0, shift))),
            Position::RIGHT | Position::LEFT => {
                (self.width, Data::new().move_to((0.0, self.width)))
            }
        };
        for bin in self.bins.iter() {
            let (x1, y1, x2, y2) = match position {
                Position::TOP => (offset, bin.height, offset + bin.width, bin.height),
                Position::RIGHT => (-bin.height, offset, -bin.height, offset - bin.width),
                Position::BOTTOM => (
                    offset,
                    shift - bin.height,
                    offset + bin.width,
                    shift - bin.height,
                ),
                Position::LEFT => (bin.height, offset, bin.height, offset - bin.width),
            };
            path_data = path_data.line_to((x1, y1)).line_to((x2, y2));
            match position {
                Position::BOTTOM | Position::TOP => {
                    offset += bin.width;
                }
                Position::LEFT | Position::RIGHT => {
                    offset -= bin.width;
                }
            };
        }
        path_data = match position {
            Position::TOP | Position::BOTTOM => path_data.line_to((offset, shift)),
            Position::RIGHT | Position::LEFT => path_data.line_to((0.0, offset)),
        };

        path_data
    }
}
