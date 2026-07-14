use crate::inference::types::InferenceError;
use serde_json::Value;

const DEFAULT_MAX_RECORD_BYTES: usize = 1024 * 1024;
const DEFAULT_MAX_BUFFER_BYTES: usize = DEFAULT_MAX_RECORD_BYTES;

pub struct NdjsonDecoder {
    buffer: Vec<u8>,
    max_record_bytes: usize,
    max_buffer_bytes: usize,
}

impl Default for NdjsonDecoder {
    fn default() -> Self {
        Self {
            buffer: Vec::new(),
            max_record_bytes: DEFAULT_MAX_RECORD_BYTES,
            max_buffer_bytes: DEFAULT_MAX_BUFFER_BYTES,
        }
    }
}

impl NdjsonDecoder {
    #[cfg(test)]
    fn with_limit(max_record_bytes: usize) -> Self {
        Self {
            buffer: Vec::new(),
            max_record_bytes,
            max_buffer_bytes: max_record_bytes,
        }
    }

    #[cfg(test)]
    fn with_limits(max_record_bytes: usize, max_buffer_bytes: usize) -> Self {
        Self {
            buffer: Vec::new(),
            max_record_bytes,
            max_buffer_bytes,
        }
    }

    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<Value>, InferenceError> {
        let mut records = Vec::new();
        let mut remaining = bytes;

        while let Some(newline) = remaining.iter().position(|byte| *byte == b'\n') {
            self.append_bounded(&remaining[..newline])?;
            if self.buffer.len() > self.max_record_bytes {
                return Err(Self::record_too_large());
            }
            let line = std::mem::take(&mut self.buffer);
            if let Some(record) = Self::decode_line(&line)? {
                records.push(record);
            }
            remaining = &remaining[newline + 1..];
        }

        self.append_bounded(remaining)?;
        if self.buffer.len() > self.max_record_bytes {
            return Err(Self::record_too_large());
        }

        Ok(records)
    }

    pub fn finish(&mut self) -> Result<Vec<Value>, InferenceError> {
        if self.buffer.len() > self.max_record_bytes || self.buffer.len() > self.max_buffer_bytes {
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

    fn append_bounded(&mut self, bytes: &[u8]) -> Result<(), InferenceError> {
        if bytes.len() > self.max_buffer_bytes.saturating_sub(self.buffer.len()) {
            return Err(InferenceError::ProviderProtocol(
                "Local provider streaming buffer exceeded the safety limit".to_string(),
            ));
        }
        self.buffer.extend_from_slice(bytes);
        Ok(())
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

    #[test]
    fn rejects_aggregate_buffer_growth_before_allocating_the_full_input() {
        let mut decoder = NdjsonDecoder::with_limits(16, 4);
        assert!(decoder.push(b"123").unwrap().is_empty());
        assert!(matches!(
            decoder.push(b"45"),
            Err(InferenceError::ProviderProtocol(message))
                if message.contains("buffer exceeded")
        ));
        assert_eq!(decoder.buffer, b"123");
    }

    #[test]
    fn processes_many_bounded_records_without_accumulating_the_input_chunk() {
        let mut decoder = NdjsonDecoder::with_limits(8, 8);
        let records = decoder.push(b"{\"a\":1}\n{\"b\":2}\n{\"c\":3}\n").unwrap();
        assert_eq!(records.len(), 3);
        assert!(decoder.buffer.is_empty());
    }
}
