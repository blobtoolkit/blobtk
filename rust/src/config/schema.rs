use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedPathConfig {
    #[serde(default)]
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub local_path: Option<PathBuf>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnnotationSourceConfig {
    pub source: ResolvedPathConfig,
    #[serde(default)]
    pub assign_to: Vec<String>,
    #[serde(default)]
    pub fields: HashMap<String, String>,
    #[serde(default)]
    pub algs: Option<Vec<crate::parse::busco::AlgConfig>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SequenceMetadataConfig {
    pub report: ResolvedPathConfig,
    #[serde(default)]
    pub metadata: HashMap<String, AnnotationSourceConfig>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WindowingConfig {
    pub lines_per_unit: usize,
    #[serde(default)]
    pub windows: Vec<crate::parse::bed::WindowSpec>,
    #[serde(default)]
    pub files: Vec<crate::parse::bed::BedConfig>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DerivedMetricConfig {
    pub name: String,
    pub target: String,
    pub source: String,
    #[serde(default)]
    pub anchor: Option<String>,
    #[serde(default)]
    pub output_type: Option<String>,
    #[serde(default)]
    pub flags: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StagedImportConfig {
    pub assembly: crate::import::AssemblyImportConfig,
    pub es: crate::import::EsConfig,
    pub sequence: SequenceMetadataConfig,
    #[serde(default)]
    pub annotations: HashMap<String, AnnotationSourceConfig>,
    pub windowing: WindowingConfig,
    #[serde(default)]
    pub derived_metrics: Vec<DerivedMetricConfig>,
    #[serde(default)]
    pub import: Option<crate::import::ImportOptions>,
}
