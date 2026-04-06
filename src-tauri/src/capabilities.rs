use serde::{Deserialize, Serialize};
use sysinfo::{System, Disks, Components, Networks};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GPUInfo {
    pub detected: bool,
    pub vendor: String,
    pub class: String,
    pub acceleration_api: String,
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
    pub worker_ready: bool,
    pub model_runtime_ready: bool,
}

#[tauri::command]
pub fn get_capabilities() -> CapabilitySummary {
    let mut sys = System::new_all();
    sys.refresh_all();

    let hostname = System::host_name().unwrap_or_else(|| "unknown".to_string());
    let cpu_brand = sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_else(|| "unknown".to_string());
    let ram_total_gb = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    
    // Simple GPU detection for Apple Silicon or basic vendors
    let os = std::env::consts::OS;
    let (gpu_detected, gpu_vendor, gpu_api) = if os == "macos" {
        (true, "apple".to_string(), "metal".to_string())
    } else {
        // Basic placeholder for Linux/Windows
        (false, "unknown".to_string(), "none".to_string())
    };

    CapabilitySummary {
        os: os.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        hostname,
        cpu_count: sys.cpus().len(),
        cpu_brand,
        ram_total_gb,
        gpu: GPUInfo {
            detected: gpu_detected,
            vendor: gpu_vendor,
            class: if os == "macos" { "integrated".to_string() } else { "unknown".to_string() },
            acceleration_api: gpu_api,
        },
        worker_ready: false, // Placeholder for Slice B
        model_runtime_ready: false, // Placeholder for Slice C
    }
}
