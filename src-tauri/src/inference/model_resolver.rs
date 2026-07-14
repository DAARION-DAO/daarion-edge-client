use crate::inference::types::{InferenceError, InferenceModelSummary};
use crate::models::registry::{CandidateModel, ModelRegistry};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedModel {
    pub canonical_model_id: String,
    pub family: String,
    pub tier: String,
    pub provider_id: String,
    pub provider_model_id: String,
}

pub struct ModelResolver {
    models: Vec<CandidateModel>,
}

impl ModelResolver {
    pub fn from_bundled_registry() -> Result<Self, InferenceError> {
        let payload = ModelRegistry::read_bundled_for_inference().map_err(|_| {
            InferenceError::Internal("Bundled model registry is unavailable".to_string())
        })?;
        Ok(Self {
            models: payload.models,
        })
    }

    pub fn resolve(&self, canonical_model_id: &str) -> Result<ResolvedModel, InferenceError> {
        let model = self
            .models
            .iter()
            .find(|model| model.id == canonical_model_id)
            .ok_or_else(|| {
                InferenceError::UnknownModel(format!(
                    "Unknown canonical model ID: {canonical_model_id}"
                ))
            })?;

        let sources = model
            .install_sources
            .iter()
            .filter(|source| source.runtime == "ollama")
            .collect::<Vec<_>>();
        if sources.len() != 1
            || sources[0].upstream_tag.trim().is_empty()
            || sources[0].upstream_tag.chars().any(char::is_whitespace)
        {
            return Err(InferenceError::UnknownModel(format!(
                "Canonical model {canonical_model_id} has no unique valid local Ollama mapping"
            )));
        }
        let source = sources[0];

        Ok(ResolvedModel {
            canonical_model_id: model.id.clone(),
            family: model.family.clone(),
            tier: model.tier.clone(),
            provider_id: "ollama".to_string(),
            provider_model_id: source.upstream_tag.clone(),
        })
    }

    pub fn summaries(
        &self,
        installed_provider_models: &HashSet<String>,
    ) -> Vec<InferenceModelSummary> {
        self.models
            .iter()
            .filter_map(|model| self.resolve(&model.id).ok())
            .map(|model| InferenceModelSummary {
                canonical_model_id: model.canonical_model_id,
                family: model.family,
                tier: model.tier,
                provider_id: model.provider_id,
                installed: installed_provider_models.contains(&model.provider_model_id),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_canonical_id_to_provider_id_without_network() {
        let resolver = ModelResolver::from_bundled_registry().unwrap();
        let model = resolver.resolve("qwen35-2b-stable").unwrap();
        assert_eq!(model.provider_model_id, "qwen3.5:2b");
        assert_eq!(model.provider_id, "ollama");
    }

    #[test]
    fn rejects_unknown_canonical_ids() {
        let resolver = ModelResolver::from_bundled_registry().unwrap();
        assert!(matches!(
            resolver.resolve("unknown:remote-tag"),
            Err(InferenceError::UnknownModel(_))
        ));
    }

    #[test]
    fn rejects_missing_duplicate_and_malformed_provider_mappings() {
        fn candidate(sources: Vec<crate::models::registry::InstallSource>) -> CandidateModel {
            CandidateModel {
                id: "fixture".to_string(),
                family: "fixture".to_string(),
                tier: "test".to_string(),
                role: "test".to_string(),
                stability: "test".to_string(),
                capabilities: Vec::new(),
                install_sources: sources,
                is_recommended: false,
            }
        }

        let source = crate::models::registry::InstallSource {
            runtime: "ollama".to_string(),
            upstream_tag: "fixture:1".to_string(),
            local_alias: "fixture".to_string(),
            estimated_download_gb: 1.0,
        };
        let resolvers = [
            ModelResolver {
                models: vec![candidate(Vec::new())],
            },
            ModelResolver {
                models: vec![candidate(vec![source.clone(), source.clone()])],
            },
            ModelResolver {
                models: vec![candidate(vec![crate::models::registry::InstallSource {
                    upstream_tag: "bad tag".to_string(),
                    ..source
                }])],
            },
        ];

        for resolver in resolvers {
            assert!(matches!(
                resolver.resolve("fixture"),
                Err(InferenceError::UnknownModel(_))
            ));
        }
    }
}
