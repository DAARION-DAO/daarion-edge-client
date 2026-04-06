use tauri::{AppHandle, Manager};
use bip39::{Mnemonic, Language};
use rand::{rngs::OsRng, RngCore};
use ed25519_dalek::{SigningKey, VerifyingKey};
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;
use hound::{WavSpec, WavWriter, SampleFormat};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use uuid::Uuid;

#[derive(serde::Serialize)]
pub struct WalletKeys {
    pub solana_pubkey: String,   // Using ed25519 for Solana as an example
    pub polygon_address: String, // Mocked EVM address
    pub base_address: String,    // Mocked EVM address
    pub mnemonic: String,
}

#[derive(serde::Serialize)]
pub struct GenesisRecord {
    pub agent_name: String,
    pub purpose: String,
    pub wallet_keys: WalletKeys,
    pub voice_signature_path: Option<String>,
}

#[tauri::command]
pub async fn generate_wallet_keys() -> Result<WalletKeys, String> {
    // Generate an actual mnemonic via bip39
    let mut rng = OsRng;
    let mut entropy = [0u8; 16];
    rng.fill_bytes(&mut entropy);
    let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
        .map_err(|e| format!("Mnemonic generation failed: {}", e))?;
    
    // Generate a simple ed25519 key for demonstration (Solana placeholder)
    let signing_key = SigningKey::generate(&mut rng);
    let verifying_key: VerifyingKey = (&signing_key).into();
    
    // EVM Mocking based on Dalek (just a simple deterministic hex string)
    let mock_evm_bytes = hex::encode(&verifying_key.as_bytes()[0..20]);
    let evm_addr = format!("0x{}", mock_evm_bytes);

    Ok(WalletKeys {
        solana_pubkey: bs58::encode(verifying_key.as_bytes()).into_string(),
        polygon_address: evm_addr.clone(),
        base_address: evm_addr,
        mnemonic: mnemonic.to_string(),
    })
}

#[tauri::command]
pub async fn record_voice_imprint(app: AppHandle, dur_secs: u64) -> Result<String, String> {
    // We launch a blocking task because CPAL streams are blocking 
    let res = tauri::async_runtime::spawn_blocking(move || {
        let host = cpal::default_host();
        let device = host.default_input_device().ok_or("No input device available")?;
        
        let config = device.default_input_config().map_err(|e| format!("Failed to get config: {}", e))?;
        
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();
        
        let spec = WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        
        let file_name = format!("voice_signature_{}.wav", Uuid::new_v4());
        let mut path = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
        path.push(&file_name);
        
        let writer = Arc::new(Mutex::new(Some(WavWriter::create(&path, spec).map_err(|e| format!("Wav error: {}", e))?)));
        let writer_clone = writer.clone();
        
        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };
        
        // Setup stream capturing
        let stream = match config.sample_format() {
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |data: &[i16], _: &_| {
                    if let Ok(mut w) = writer_clone.lock() {
                        if let Some(writer) = w.as_mut() {
                            for &sample in data.iter() {
                                let _ = writer.write_sample(sample);
                            }
                        }
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _: &_| {
                    if let Ok(mut w) = writer_clone.lock() {
                        if let Some(writer) = w.as_mut() {
                            for &sample in data.iter() {
                                let sample_i16 = (sample * std::i16::MAX as f32) as i16;
                                let _ = writer.write_sample(sample_i16);
                            }
                        }
                    }
                },
                err_fn,
                None,
            ),
            _ => return Err("Unsupported sample format".to_string()),
        }.map_err(|e| format!("Build stream error: {}", e))?;
        
        stream.play().map_err(|e| format!("Play error: {}", e))?;
        std::thread::sleep(Duration::from_secs(dur_secs));
        drop(stream);
        
        if let Ok(mut w) = writer.lock() {
            if let Some(writer) = w.take() {
                let _ = writer.finalize();
            }
        }
        
        Ok(path.to_string_lossy().to_string())
    }).await.map_err(|e| format!("Join error: {}", e))??;
    
    Ok(res)
}
