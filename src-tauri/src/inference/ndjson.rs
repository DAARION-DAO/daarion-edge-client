use crate::inference::types::InferenceError;
use serde_json::Value;

const DEFAULT_MAX_RECORD_BYTES: usize = 1024 * 1024;

pub struct NdjsonDecoder {
    buffer: Vec<u8>,
    max_record_bytes: usize,
}

impl Default for NdjsonDecoder {
    fn default() -> Self {
        Self {
            buffer: Vec::new(),
            max_record_bytes: DEFAULT_MAX_RECORD_BYTES,
        }
    }
}

impl NdjsonDecoder {
    #[cfg(test)]
    fn with_limit(max_record_bytes: usize) -> Self {
        Self {
            buffer: Vec::new(),
            max_record_bytes,
        }
    }

    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<Value>, InferenceError> {
        self.buffer.extend_from_slice(bytes);
        let mut records = Vec::new();

        while let Some(newline) = self.buffer.iter().position(|byte| *byte == b'\n') {
            if newline > self.max_record_bytes {
                return Err(Self::record_too_large());
            }
            let line: Vec<u8> = self.buffer.drain(..=newline).collect();
            if let Some(record) = Self::decode_line(&line[..line.len() - 1])? {
                records.push(record);
            }
        }

        if self.buffer.len() > self.max_record_bytes {
            return Err(Self::record_too_large());
        }

        Ok(records)
    }

    pub fn finish(&mut self) -> Result<Vec<Value>, InferenceError> {
        if self.buffer.len() > self.max_record_bytes {
            return Err(Self::record_too_large());
        }
        let final_line = std::mem::take(&mut self.buffer);
        Ok(Self::decode_line(&final_line)?.into_iter().collect())
    }

    fn decode_line(line: &[u8]) -> Result<Option<Value>, InferenceError> {
        let trimmed = line
            .iter()
            .copied()
            .skip_while(|byte| byte.is_ascii_whitespace())
            .collect::<Vec<_>>();
        let trimmed_end = trimmed
            .iter()
            .rposition(|byte| !byte.is_ascii_whitespace())
            .map(|index| &trimmed[..=index])
            .unwrap_or_default();

        if trimmed_end.is_empty() {
            return Ok(None);
        }

        serde_json::from_slice(trimmed_end).map(Some).map_err(|_| {
            InferenceError::ProviderProtocol(
                "Local provider returned a malformed streaming record".to_string(),
            )
        })
    }

    fn record_too_large() -> InferenceError {
        InferenceError::ProviderProtocol(
            "Local provider streaming record exceeded the safety limit".to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffers_split_json_and_utf8() {
        let payload = "{\"message\":{\"content\":\"Привіт\"}}\n".as_bytes();
        let split = payload.iter().position(|byte| *byte >= 0x80).unwrap() + 1;
        let mut decoder = NdjsonDecoder::default();
        assert!(decoder.push(&payload[..split]).unwrap().is_empty());
        let records = decoder.push(&payload[split..]).unwrap();
        assert_eq!(records[0]["message"]["content"], "Привіт");
    }

    #[test]
    fn handles_multiple_blank_and_final_records() {
        let mut decoder = NdjsonDecoder::default();
        let records = decoder.push(b"\n{\"a\":1}\n {\"b\":2}\r\n").unwrap();
        assert_eq!(records.len(), 2);
        assert!(decoder.push(b"{\"done\":true}").unwrap().is_empty());
        assert_eq!(decoder.finish().unwrap()[0]["done"], true);
    }

    #[test]
    fn rejects_malformed_and_oversized_records() {
        let mut malformed = NdjsonDecoder::default();
        assert!(malformed.push(b"not-json\n").is_err());

        let mut oversized = NdjsonDecoder::with_limit(4);
        assert!(oversized.push(b"12345").is_err());
    }
}
