use crate::config::paths::resolve_source_path;
use crate::config::schema::{ResolvedPathConfig, StagedImportConfig};

pub fn validate_staged_import_config(staged: &StagedImportConfig) -> Result<(), anyhow::Error> {
    if staged.sequence.report.path.is_none() && staged.sequence.report.local_path.is_none() {
        return Err(anyhow::anyhow!(
            "staged import config is missing a sequence report path or local_path"
        ));
    }
    resolve_source_path(&staged.sequence.report)?;

    for file in &staged.windowing.files {
        let resolved = resolve_source_path(&ResolvedPathConfig {
            path: Some(file.path.clone()),
            local_path: file.local_path.clone(),
        })?;
        let _ = resolved;
    }

    for annotation in staged.annotations.values() {
        resolve_source_path(&annotation.source)?;
    }

    if staged.windowing.files.is_empty() {
        return Err(anyhow::anyhow!(
            "staged import config is missing any window bed files"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::{
        AnnotationSourceConfig, ResolvedPathConfig, SequenceMetadataConfig,
    };
    use std::collections::HashMap;

    #[test]
    fn rejects_missing_sequence_report() {
        let staged = StagedImportConfig {
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
            sequence: SequenceMetadataConfig {
                report: ResolvedPathConfig {
                    path: None,
                    local_path: None,
                },
                metadata: HashMap::new(),
            },
            annotations: HashMap::from_iter([(
                "annotation".to_string(),
                AnnotationSourceConfig {
                    source: ResolvedPathConfig {
                        path: Some(std::path::PathBuf::from("/tmp/annotation.bed.gz")),
                        local_path: None,
                    },
                    assign_to: vec!["window".to_string()],
                    fields: HashMap::new(),
                    algs: None,
                },
            )]),
            windowing: crate::config::schema::WindowingConfig {
                lines_per_unit: 1000,
                windows: vec![],
                files: vec![crate::parse::bed::BedConfig {
                    path: std::path::PathBuf::from("/tmp/window.bed.gz"),
                    local_path: None,
                    value_columns: vec![],
                }],
            },
            derived_metrics: vec![],
            import: None,
        };

        assert!(validate_staged_import_config(&staged).is_err());
    }
}
