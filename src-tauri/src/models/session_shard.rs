use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionShard {
    pub shard_id: u32,
    pub shard_name: String,
}

pub struct ShardDeriver;

impl ShardDeriver {
    /// Derives a stable shard ID from a session_id.
    /// Uses a simple stable hash (similar to FNV-1a) for the skeleton.
    pub fn derive_shard(session_id: &str, shard_count: u32) -> u32 {
        if shard_count == 0 { return 0; }
        
        let mut hash: u32 = 2166136261;
        for b in session_id.as_bytes() {
            hash ^= *b as u32;
            hash = hash.wrapping_mul(16777619);
        }
        
        hash % shard_count
    }

    pub fn get_shard_name(shard_id: u32) -> String {
        format!("shard-{}", shard_id)
    }
}
