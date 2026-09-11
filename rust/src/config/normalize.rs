use std::collections::HashMap;

use crate::config::schema::StagedImportConfig;

/// Normalize the staged config into a consistent runtime shape before validation.
///
/// This lives in the config subsystem rather than the legacy compatibility layer,
/// so the runtime can evolve without a large ad hoc schema surface.
pub fn normalize_staged_import_config(staged: &mut StagedImportConfig) {
    if staged.sequence.metadata.is_empty() {
        staged.sequence.metadata = HashMap::new();
    }

    if staged.annotations.is_empty() {
        staged.annotations = HashMap::new();
    }

    for annotation in staged.annotations.values_mut() {
        if annotation.assign_to.is_empty() {
            annotation.assign_to = vec!["sequence".to_string()];
        }
    }
}

#[deprecated(
    note = "compatibility shim for legacy config; prefer config::normalize::normalize_staged_import_config"
)]
pub fn normalize_legacy_staged_config(staged: &mut StagedImportConfig) {
    normalize_staged_import_config(staged);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::AnnotationSourceConfig;
    use crate::config::schema::ResolvedPathConfig;

    #[test]
    fn adds_default_assignment_for_unassigned_annotation() {
        let mut staged = StagedImportConfig {
            assembly: crate::import::AssemblyImportConfig {
                accession: "GCA_00000001.1".to_string(),
                taxon_id: None,
                ancestors: vec![],
                lineage: vec![],
            },
            es: crate::import::EsConfig {
                host: "http://localhost".to_string(),
                port: 9200,
                username: None,
                password: None,
                hub: crate::import::HubConfig {
                    name: "goat".to_string(),
                    release: "test".to_string(),
                    taxonomy: "ncbi".to_string(),
                },
            },
            sequence: crate::config::schema::SequenceMetadataConfig {
                report: ResolvedPathConfig {
                    path: Some(std::path::PathBuf::from("/tmp/report.jsonl")),
                    local_path: None,
                },
                metadata: HashMap::new(),
            },
            annotations: HashMap::from_iter([(
                "custom".to_string(),
                AnnotationSourceConfig {
                    source: ResolvedPathConfig {
                        path: Some(std::path::PathBuf::from("/tmp/custom.bed.gz")),
                        local_path: None,
                    },
                    assign_to: vec![],
                    fields: HashMap::new(),
                    algs: None,
                },
            )]),
            windowing: crate::config::schema::WindowingConfig {
                lines_per_unit: 1000,
                windows: vec![],
                files: vec![],
            },
            derived_metrics: vec![],
            import: None,
        };

        normalize_staged_import_config(&mut staged);
        assert_eq!(staged.annotations["custom"].assign_to, vec!["sequence"]);
    }
}
