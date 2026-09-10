//! One-shot managed-device observation. No task runner, inbound listener or inference.
#![allow(dead_code)]
use daarion_edge_client_lib::{capabilities, identity};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{path::PathBuf, time::Duration};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    node_id: String,
    audience: String,
    endpoint: String,
    ca_file: PathBuf,
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    if !cfg!(target_os = "macos") {
        return Err("Managed node CLI currently supports macOS only".into());
    }
    let args: Vec<_> = std::env::args().collect();
    let home = PathBuf::from(std::env::var("HOME")?);
    let app_dir = home.join("Library/Application Support/city.daarion.edge");
    if args.len() == 2 && args[1] == "init" {
        println!(
            "{}",
            serde_json::to_string(&identity::node_identity_at(&app_dir)?)?
        );
        return Ok(());
    }
    if args.len() != 3 || args[1] != "observe" {
        return Err("Usage: daarion-node init | observe CONFIG".into());
    }
    let config: Config = serde_json::from_slice(&std::fs::read(&args[2])?)?;
    let url = reqwest::Url::parse(&config.endpoint)?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/node/observations"
        || config.node_id.is_empty()
        || config.node_id.len() > 128
        || config.audience.is_empty()
    {
        return Err("Invalid managed observation configuration".into());
    }
    // Observe never creates identity implicitly.
    if !app_dir.join(identity::IDENTITY_FILE).is_file() {
        return Err("Initialize device identity explicitly first".into());
    }
    let device = identity::node_identity_at(&app_dir)?;
    let caps = capabilities::get_capabilities();
    let local = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(5))
        .build()?;
    async fn probe(client: &reqwest::Client, path: &str) -> Option<Value> {
        let result = client
            .get(format!("http://127.0.0.1:11434{path}"))
            .send()
            .await
            .ok()?;
        if !result.status().is_success() {
            return None;
        }
        let bytes = result.bytes().await.ok()?;
        if bytes.len() > 262144 {
            return None;
        }
        serde_json::from_slice(&bytes).ok()
    }
    let version = probe(&local, "/api/version").await;
    let tags = probe(&local, "/api/tags").await;
    let active = probe(&local, "/api/ps").await;
    let facts = json!({"platform":caps.os,"architecture":caps.arch,"cpu":caps.cpu_brand,
        "cpu_count":caps.cpu_count,"ram_bytes":(caps.ram_total_gb*1073741824.0) as u64,
        "accelerator":caps.gpu.acceleration_api,
        "ollama":{"version":version.as_ref().and_then(|v|v.get("version")).and_then(Value::as_str),
          "installed_model_count":tags.as_ref().and_then(|v|v.get("models")).and_then(Value::as_array).map(Vec::len),
          "loaded_model_count":active.as_ref().and_then(|v|v.get("models")).and_then(Value::as_array).map(Vec::len)},
        "worker_execution":"NOT_ENABLED","inference_probe":"NOT_PERFORMED"});
    let digest = hex::encode(Sha256::digest(serde_json::to_vec(&facts)?));
    let payload = json!({"schema":"daarion.node.observation.v1","node_id":config.node_id,
        "device_id":device.node_id,"audience":config.audience,"observation_id":uuid::Uuid::new_v4().to_string(),
        "observed_at":chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis,true),
        "capability_digest":digest,"facts":facts});
    let canonical = format!(
        "daarion.node.observation.v1\n{}",
        serde_json::to_string(&payload)?
    );
    let signature = identity::sign_node_observation_at(&app_dir, &canonical)?;
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(15))
        .add_root_certificate(reqwest::Certificate::from_pem(&std::fs::read(
            config.ca_file,
        )?)?)
        .build()?;
    let response = client
        .post(url)
        .json(&json!({"payload":payload,"signature":signature}))
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(format!("Observation rejected: HTTP {}", response.status()).into());
    }
    println!("{}", response.text().await?);
    Ok(())
}
#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("Node observation failed: {error}");
        std::process::exit(1);
    }
}
