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
pub struct ScatterAxis {
    pub label: String,
    pub scale: String,
    pub domain: [f64; 2],
    pub range: [f64; 2],
    pub clamp: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct ScatterData {
    pub points: Vec<ScatterPoint>,
    pub x: ScatterAxis,
    pub y: ScatterAxis,
    pub z: ScatterAxis,
    pub categories: Vec<Category>,
}
