use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct TickOptions {
    pub font_size: f64,
    pub font_color: String,
    pub show_minor_tick: bool,
    pub weight: f64,
    pub length: f64,
    pub rotate: f64,
    pub status: TickStatus,
}

impl Default for TickOptions {
    fn default() -> TickOptions {
        TickOptions {
            font_size: 20.0,
            font_color: "black".to_string(),
            show_minor_tick: false,
            weight: 2.0,
            length: 10.0,
            rotate: 0.0,
            status: TickStatus::Major,
        }
    }
}

#[derive(Clone, Debug)]
pub enum AxisName {
    X,
    Y,
    Z,
    Cat,
}

#[derive(Clone, Debug)]
pub enum Position {
    TOP,
    RIGHT,
    BOTTOM,
    LEFT,
}

impl FromStr for AxisName {
    type Err = ();
    fn from_str(input: &str) -> Result<AxisName, Self::Err> {
        match input {
            "x" => Ok(AxisName::X),
            "y" => Ok(AxisName::Y),
            "z" => Ok(AxisName::Z),
            "cat" => Ok(AxisName::Cat),
            _ => Err(()),
        }
    }
}

impl FromStr for Position {
    type Err = ();
    fn from_str(input: &str) -> Result<Position, Self::Err> {
        match input {
            "top" => Ok(Position::TOP),
            "right" => Ok(Position::RIGHT),
            "bottom" => Ok(Position::BOTTOM),
            "left" => Ok(Position::LEFT),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Scale {
    LINEAR,
    SQRT,
    LOG,
}

impl FromStr for Scale {
    type Err = ();
    fn from_str(input: &str) -> Result<Scale, Self::Err> {
        match input {
            "scaleLinear" => Ok(Scale::LINEAR),
            "scaleSqrt" => Ok(Scale::SQRT),
            "scaleLog" => Ok(Scale::LOG),
            _ => Err(()),
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
pub struct AxisOptions {
    pub label: String,
    pub position: Position,
    pub padding: [f64; 2],
    pub offset: f64,
    pub font_size: f64,
    pub weight: f64,
    pub color: String,
    pub scale: Scale,
    pub domain: [f64; 2],
    pub range: [f64; 2],
    pub clamp: Option<f64>,
    pub major_ticks: TickOptions,
    pub minor_ticks: TickOptions,
}

impl Default for AxisOptions {
    fn default() -> AxisOptions {
        AxisOptions {
            label: "".to_string(),
            position: Position::LEFT,
            padding: [0.0, 0.0],
            offset: 0.0,
            font_size: 30.0,
            weight: 3.0,
            color: "black".to_string(),
            scale: Scale::LINEAR,
            domain: [0.0, 1.0],
            range: [0.0, 100.0],
            clamp: None,
            major_ticks: TickOptions {
                ..Default::default()
            },
            minor_ticks: TickOptions {
                status: TickStatus::Minor,
                weight: 1.0,
                length: 5.0,
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, Debug)]
pub enum TickStatus {
    Major,
    Minor,
}

impl FromStr for TickStatus {
    type Err = ();
    fn from_str(input: &str) -> Result<TickStatus, Self::Err> {
        match input {
            "major" => Ok(TickStatus::Major),
            "minor" => Ok(TickStatus::Minor),
            _ => Err(()),
        }
    }
}
