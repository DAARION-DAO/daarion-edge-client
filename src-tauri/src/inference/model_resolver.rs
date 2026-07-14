use crate::inference::types::{InferenceError, InferenceModelSummary};
use crate::models::registry::{CandidateModel, ModelRegistry};
use std::collections::HashSet;

const MAX_PROVIDER_MODEL_ID_BYTES: usize = 256;

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
        let models = self
            .models
            .iter()
            .filter(|model| model.id == canonical_model_id)
            .collect::<Vec<_>>();
        if models.len() != 1 {
            return Err(InferenceError::UnknownModel(
                "Canonical model ID is not uniquely available".to_string(),
            ));
        }
        let model = models[0];

        let sources = model
            .install_sources
            .iter()
            .filter(|source| source.runtime == "ollama")
            .collect::<Vec<_>>();
        if sources.len() != 1 || !Self::is_valid_ollama_tag(&sources[0].upstream_tag) {
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

    fn is_valid_ollama_tag(candidate: &str) -> bool {
        if candidate.is_empty()
            || candidate.len() > MAX_PROVIDER_MODEL_ID_BYTES
            || !candidate.is_ascii()
        {
            return false;
        }

        let mut parts = candidate.split(':');
        let Some(name) = parts.next() else {
            return false;
        };
        let tag = parts.next();
        if parts.next().is_some() {
            return false;
        }

        fn valid_segment(segment: &str) -> bool {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
                && segment
                    .as_bytes()
                    .first()
                    .is_some_and(u8::is_ascii_alphanumeric)
                && segment
                    .as_bytes()
                    .last()
                    .is_some_and(u8::is_ascii_alphanumeric)
        }

        name.split('/').all(valid_segment) && tag.is_none_or(valid_segment)
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
    fn rejects_duplicate_canonical_model_entries() {
        let resolver = ModelResolver::from_bundled_registry().unwrap();
        let model = resolver
            .models
            .iter()
            .find(|model| model.id == "qwen35-2b-stable")
            .unwrap()
            .clone();
        let duplicate = ModelResolver {
            models: vec![model.clone(), model],
        };

        assert!(matches!(
            duplicate.resolve("qwen35-2b-stable"),
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

    #[test]
    fn rejects_provider_tags_outside_the_bounded_ollama_grammar() {
        for candidate in [
            "",
            " bad",
            "bad ",
            "https://example.com/model:tag",
            "model::tag",
            "/model:tag",
            "model/:tag",
            "model:@tag",
            "model:tag/other",
            "model:tag?remote=true",
        ] {
            assert!(
                !ModelResolver::is_valid_ollama_tag(candidate),
                "accepted malformed tag {candidate:?}"
            );
        }

        for candidate in ["qwen3.5:2b", "namespace/model-name:latest", "model_name"] {
            assert!(
                ModelResolver::is_valid_ollama_tag(candidate),
                "rejected valid tag {candidate:?}"
            );
        }

        assert!(!ModelResolver::is_valid_ollama_tag(
            &"x".repeat(MAX_PROVIDER_MODEL_ID_BYTES + 1)
        ));
    }
}
