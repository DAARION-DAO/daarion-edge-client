use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GPUInfo {
    pub detected: bool,
    pub vendor: String,
    pub class: String,
    pub acceleration_api: String,
}

/// Device form-factor category inferred from hardware
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DeviceClass {
    Smartphone,   // RAM < 6 GB, likely ARM, high-efficiency needed
    Tablet,       // RAM 6–11 GB, ARM or x86
    Laptop,       // RAM 12–23 GB, x86 or Apple Silicon
    Workstation,  // RAM ≥ 24 GB, full precision possible
}

/// Recommended GGUF model for this device
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecommendedModel {
    pub model_id: String,        // canonical id
    pub display_name: String,    // human-readable
    pub params: String,          // "0.4B", "2B", etc.
    pub quantization: String,    // "Q4_0", "Q8_0", etc.
    pub size_gb: f32,            // approx download size
    pub context_tokens: u32,     // max context window
    pub reason: String,          // why this was selected
    pub download_url: String,    // HuggingFace GGUF direct link
    pub performance_tier: String, // "ultra-lite", "lite", "balanced", "full"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CapabilitySummary {
    pub os: String,
    pub arch: String,
    pub hostname: String,
    pub cpu_count: usize,
    pub cpu_brand: String,
    pub ram_total_gb: f64,
    pub gpu: GPUInfo,
    pub device_class: DeviceClass,
    pub recommended_model: RecommendedModel,
    pub alternative_models: Vec<RecommendedModel>,
    pub worker_ready: bool,
    pub model_runtime_ready: bool,
}

/// Select the best model based on RAM + OS + architecture
fn select_model(ram_gb: f64, os: &str, arch: &str) -> (DeviceClass, RecommendedModel, Vec<RecommendedModel>) {
    // Determine device class
    let device_class = match (ram_gb, os, arch) {
        (r, _, "aarch64") if r < 6.0 => DeviceClass::Smartphone,
        (r, _, "aarch64") if r < 12.0 => DeviceClass::Tablet,
        (r, _, _) if r < 6.0 => DeviceClass::Smartphone,
        (r, _, _) if r < 12.0 => DeviceClass::Tablet,
        (r, _, _) if r < 24.0 => DeviceClass::Laptop,
        _ => DeviceClass::Workstation,
    };

    // Model catalogue — DAARION Approved List
    let gemma4_tiny = RecommendedModel {
        model_id: "gemma-4-it-qat-q4_0".to_string(),
        display_name: "Gemma 4 Tiny".to_string(),
        params: "0.4B".to_string(),
        quantization: "Q4_0".to_string(),
        size_gb: 0.3,
        context_tokens: 4096,
        reason: String::new(),
        download_url: "https://huggingface.co/bartowski/gemma-4-it-GGUF/resolve/main/gemma-4-it-Q4_0.gguf".to_string(),
        performance_tier: "ultra-lite".to_string(),
    };

    let qwen35_08b = RecommendedModel {
        model_id: "qwen3.5-0.8b-it-q4_k_m".to_string(),
        display_name: "Qwen 3.5".to_string(),
        params: "0.8B".to_string(),
        quantization: "Q4_K_M".to_string(),
        size_gb: 0.5,
        context_tokens: 8192,
        reason: String::new(),
        download_url: "https://huggingface.co/Qwen/Qwen3.5-0.8B-GGUF/resolve/main/Qwen3.5-0.8B-Instruct-Q4_K_M.gguf".to_string(),
        performance_tier: "lite".to_string(),
    };

    let gemma4_2b = RecommendedModel {
        model_id: "gemma-4-2b-it-q4_k_m".to_string(),
        display_name: "Gemma 4".to_string(),
        params: "2B".to_string(),
        quantization: "Q4_K_M".to_string(),
        size_gb: 1.5,
        context_tokens: 8192,
        reason: String::new(),
        download_url: "https://huggingface.co/bartowski/gemma-2-2b-it-GGUF/resolve/main/gemma-2-2b-it-Q4_K_M.gguf".to_string(),
        performance_tier: "balanced".to_string(),
    };

    let qwen35_2b = RecommendedModel {
        model_id: "qwen3.5-2b-it-q8_0".to_string(),
        display_name: "Qwen 3.5".to_string(),
        params: "2B".to_string(),
        quantization: "Q8_0".to_string(),
        size_gb: 2.1,
        context_tokens: 16384,
        reason: String::new(),
        download_url: "https://huggingface.co/Qwen/Qwen3.5-2B-GGUF/resolve/main/Qwen3.5-2B-Instruct-Q8_0.gguf".to_string(),
        performance_tier: "balanced".to_string(),
    };

    let gemma4_4b_q8 = RecommendedModel {
        model_id: "gemma-4-4b-it-q8_0".to_string(),
        display_name: "Gemma 4".to_string(),
        params: "4B".to_string(),
        quantization: "Q8_0".to_string(),
        size_gb: 4.5,
        context_tokens: 32768,
        reason: String::new(),
        download_url: "https://huggingface.co/bartowski/gemma-2-9b-it-GGUF/resolve/main/gemma-2-9b-it-Q8_0.gguf".to_string(),
        performance_tier: "full".to_string(),
    };

    let qwen35_9b = RecommendedModel {
        model_id: "qwen3.5-9b-it-q8_0".to_string(),
        display_name: "Qwen 3.5".to_string(),
        params: "9B".to_string(),
        quantization: "Q8_0".to_string(),
        size_gb: 9.5,
        context_tokens: 32768,
        reason: String::new(),
        download_url: "https://huggingface.co/Qwen/Qwen3.5-9B-GGUF/resolve/main/Qwen3.5-9B-Instruct-Q8_0.gguf".to_string(),
        performance_tier: "full".to_string(),
    };

    let (primary_reason, primary, alternatives) = match device_class {
        DeviceClass::Smartphone => {
            if ram_gb < 3.0 {
                (
                    format!("Ultra-lite model optimized for {:.0}GB RAM mobile device. Minimal battery drain.", ram_gb),
                    gemma4_tiny.clone(),
                    vec![qwen35_08b.clone()],
                )
            } else {
                (
                    format!("Lite model for {:.0}GB smartphone. Balanced speed and quality.", ram_gb),
                    qwen35_08b.clone(),
                    vec![gemma4_tiny, gemma4_2b.clone()],
                )
            }
        }
        DeviceClass::Tablet => {
            (
                format!("Balanced model for {:.0}GB tablet. Good context window, smooth inference.", ram_gb),
                gemma4_2b.clone(),
                vec![qwen35_08b, qwen35_2b.clone()],
            )
        }
        DeviceClass::Laptop => {
            if os == "macos" {
                (
                    format!("Full-precision model for Apple Silicon ({:.0}GB RAM). Metal acceleration enabled.", ram_gb),
                    qwen35_2b.clone(),
                    vec![gemma4_2b, gemma4_4b_q8.clone()],
                )
            } else {
                (
                    format!("Balanced model for {:.0}GB laptop. Q8 precision for best quality/speed ratio.", ram_gb),
                    qwen35_2b.clone(),
                    vec![gemma4_2b, gemma4_4b_q8.clone()],
                )
            }
        }
        DeviceClass::Workstation => {
            (
                format!("Full model for workstation ({:.0}GB RAM). Maximum context and reasoning quality.", ram_gb),
                qwen35_9b.clone(),
                vec![gemma4_4b_q8, qwen35_2b],
            )
        }
    };

    let primary = RecommendedModel {
        reason: primary_reason,
        ..primary
    };

    (device_class, primary, alternatives)
}

/// Detect GPU capabilities based on OS/arch
fn detect_gpu(os: &str, arch: &str) -> GPUInfo {
    match os {
        "macos" => GPUInfo {
            detected: true,
            vendor: "Apple".to_string(),
            class: "Unified Memory (Metal)".to_string(),
            acceleration_api: "Metal".to_string(),
        },
        "linux" => {
            // Check for NVIDIA/AMD via /proc or env
            GPUInfo {
                detected: std::env::var("CUDA_VISIBLE_DEVICES").is_ok()
                    || std::path::Path::new("/dev/nvidia0").exists(),
                vendor: if std::path::Path::new("/dev/nvidia0").exists() {
                    "NVIDIA".to_string()
                } else {
                    "Unknown".to_string()
                },
                class: "Discrete".to_string(),
                acceleration_api: if std::path::Path::new("/dev/nvidia0").exists() {
                    "CUDA".to_string()
                } else {
                    "CPU".to_string()
                },
            }
        }
        "android" => GPUInfo {
            detected: true,
            vendor: "Qualcomm/ARM".to_string(),
            class: "Mobile GPU".to_string(),
            acceleration_api: if arch == "aarch64" { "Vulkan".to_string() } else { "OpenGL ES".to_string() },
        },
        _ => GPUInfo {
            detected: false,
            vendor: "Unknown".to_string(),
            class: "Unknown".to_string(),
            acceleration_api: "CPU".to_string(),
        },
    }
}

#[tauri::command]
pub fn get_capabilities() -> CapabilitySummary {
    let mut sys = System::new_all();
    sys.refresh_all();

    let hostname = System::host_name().unwrap_or_else(|| "sovereign-node".to_string());
    let cpu_brand = sys
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let ram_total_gb = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;

    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let gpu = detect_gpu(os, arch);
    let (device_class, recommended_model, alternative_models) =
        select_model(ram_total_gb, os, arch);

    CapabilitySummary {
        os: os.to_string(),
        arch: arch.to_string(),
        hostname,
        cpu_count: sys.cpus().len(),
        cpu_brand,
        ram_total_gb,
        gpu,
        device_class,
        recommended_model,
        alternative_models,
        worker_ready: false,
        model_runtime_ready: false,
    }
}
