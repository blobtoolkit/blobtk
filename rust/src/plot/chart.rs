use svg::node::element::{Circle, Group};

use super::{
    axis::ChartAxes,
    component::chart_axis,
    data::{HistogramData, ScatterData},
    style::{path_filled, path_open},
};

#[derive(Clone, Debug)]
pub struct Dimensions {
    pub height: f64,
    pub width: f64,
    pub margin: [f64; 4],
    pub padding: [f64; 4],
}

impl Default for Dimensions {
    fn default() -> Dimensions {
        Dimensions {
            height: 900.0,
            width: 900.0,
            margin: [10.0, 10.0, 100.0, 100.0],
            padding: [50.0, 50.0, 50.0, 50.0],
        }
    }
}

#[derive(Clone, Debug)]
pub struct Chart {
    pub axes: ChartAxes,
    pub scatter_data: Option<ScatterData>,
    // pub bar_data: Option<BarData>,
    // pub line_data: Option<LineData>,
    pub histogram_data: Option<Vec<HistogramData>>,
    pub scatter_options: Vec<(String, String)>,
    pub histogram_options: Vec<(String, String)>,
    pub dimensions: Dimensions,
}

impl Default for Chart {
    fn default() -> Chart {
        Chart {
            axes: ChartAxes {
                x: None,
                y: None,
                x2: None,
                y2: None,
            },
            scatter_data: None,
            histogram_data: None,
            scatter_options: vec![],
            histogram_options: vec![],
            dimensions: Dimensions {
                ..Default::default()
            },
        }
    }
}

impl Chart {
    pub fn svg(self) -> Group {
        let opacity = 0.6;
        let mut group = Group::new();
        if self.scatter_data.is_some() {
            let scatter_data = self.scatter_data.unwrap();
            let mut scatter_group = Group::new();
            for point in scatter_data.points.iter() {
                scatter_group = scatter_group.add(
                    Circle::new()
                        .set("cx", point.x)
                        .set("cy", point.y)
                        .set("r", point.z)
                        .set("fill", point.color.clone().unwrap())
                        .set("stroke", "#999999")
                        .set("fill-opacity", opacity),
                );
            }
            group = group.add(scatter_group.set(
                "transform",
                format!(
                    "translate({}, {})",
                    self.dimensions.padding[3], self.dimensions.padding[2]
                ),
            ));
        }
        if self.histogram_data.is_some() {
            let mut hist_group = Group::new();
            let mut hist_paths = vec![];
            for hist in self.histogram_data.unwrap() {
                let color;
                color = hist.category.clone().unwrap().color;
                let path_data = hist
                    .clone()
                    .to_path_data(self.axes.x.clone().unwrap().position, true);
                hist_group = hist_group
                    .add(path_filled(path_data.clone(), Some(&color)).set("opacity", opacity));
                hist_paths.push((path_data, color));
            }
            for (path, color) in hist_paths {
                hist_group = hist_group.add(path_open(path, Some(&color), Some(2.0)));
            }
            group = group.add(hist_group.set(
                "transform",
                format!(
                    "translate({}, {})",
                    self.dimensions.padding[3], self.dimensions.padding[2]
                ),
            ));
        }

        if self.axes.x.is_some() {
            group = group.add(chart_axis(&self.axes.x.unwrap()));
        }
        if self.axes.y.is_some() {
            group = group.add(chart_axis(&self.axes.y.unwrap()));
        }
        if self.axes.x2.is_some() {
            group = group.add(chart_axis(&self.axes.x2.unwrap()));
        }
        if self.axes.y2.is_some() {
            group = group.add(chart_axis(&self.axes.y2.unwrap()));
        }
        // let x_axis = chart_axis(&scatter_data.x);
        // let y_axis = chart_axis(&scatter_data.y);

        group
    }
}
