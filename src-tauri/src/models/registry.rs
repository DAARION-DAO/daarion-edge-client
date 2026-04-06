use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelRegistryEntry {
    pub model_id: String,
    pub name: String,
    pub family: String,
    pub quantization: String,
    pub size_gb: f32,
    pub artifact_hash: String, // SHA-256
    pub runtime: String, // e.g., "llama.cpp"
    pub min_ram_gb: u32,
}

pub struct ModelRegistry;

impl ModelRegistry {
    pub fn get_supported_models() -> Vec<ModelRegistryEntry> {
        vec![
            ModelRegistryEntry {
                model_id: "qwen2.5-0.5b-instruct".to_string(),
                name: "Qwen 2.5 0.5B Instruct".to_string(),
                family: "qwen".to_string(),
                quantization: "Q4_K_M".to_string(),
                size_gb: 0.4,
                artifact_hash: "5cc7d020d888f615e98214227914757c913501a39fd64a938c5d14dfb0744fc3".to_string(), // Dummy for M1
                runtime: "llama.cpp".to_string(),
                min_ram_gb: 1,
            },
            ModelRegistryEntry {
                model_id: "stable-embedding-v1".to_string(),
                name: "Stable Embedding v1".to_string(),
                family: "bert".to_string(),
                quantization: "F16".to_string(),
                size_gb: 0.1,
                artifact_hash: "b0744fc35cc7d020d888f615e98214227914757c913501a39fd64a938c5d14df".to_string(), // Dummy for M1
                runtime: "onnx".to_string(),
                min_ram_gb: 1,
            }
        ]
    }
}
