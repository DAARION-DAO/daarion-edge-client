use crate::inference::types::InferenceError;
use url::{Host, Url};

pub const DEFAULT_OLLAMA_ENDPOINT: &str = "http://127.0.0.1:11434/";

#[derive(Debug, Clone)]
pub struct LocalEndpoint {
    base: Url,
}

impl LocalEndpoint {
    pub fn parse(candidate: &str) -> Result<Self, InferenceError> {
        let parsed = Url::parse(candidate).map_err(|_| {
            InferenceError::PolicyViolation("Local provider endpoint is invalid".to_string())
        })?;

        if parsed.scheme() != "http"
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.query().is_some()
            || parsed.fragment().is_some()
            || parsed.path() != "/"
        {
            return Err(InferenceError::PolicyViolation(
                "Local provider endpoint must be a plain loopback HTTP origin".to_string(),
            ));
        }

        let port = parsed.port().ok_or_else(|| {
            InferenceError::PolicyViolation(
                "Local provider endpoint must include a port".to_string(),
            )
        })?;

        let host = parsed.host().ok_or_else(|| {
            InferenceError::PolicyViolation("Local provider endpoint has no host".to_string())
        })?;

        let normalized_host = match host {
            Host::Ipv4(address) if address.is_loopback() => address.to_string(),
            Host::Ipv6(address) if address.is_loopback() => format!("[{address}]"),
            Host::Domain(domain) if domain.eq_ignore_ascii_case("localhost") => {
                "127.0.0.1".to_string()
            }
            _ => {
                return Err(InferenceError::PolicyViolation(
                    "Remote inference endpoints are forbidden by LocalOnly policy".to_string(),
                ))
            }
        };

        let base = Url::parse(&format!("http://{normalized_host}:{port}/")).map_err(|_| {
            InferenceError::PolicyViolation(
                "Local provider endpoint could not be normalized".to_string(),
            )
        })?;

        Ok(Self { base })
    }

    pub fn default_ollama() -> Result<Self, InferenceError> {
        Self::parse(DEFAULT_OLLAMA_ENDPOINT)
    }

    pub fn join(&self, path: &str) -> Result<Url, InferenceError> {
        self.base.join(path).map_err(|_| {
            InferenceError::Internal("Local provider route could not be constructed".to_string())
        })
    }

    pub fn display(&self) -> String {
        self.base.as_str().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_and_normalizes_loopback_origins() {
        assert_eq!(
            LocalEndpoint::parse("http://localhost:11434/")
                .unwrap()
                .display(),
            DEFAULT_OLLAMA_ENDPOINT
        );
        assert_eq!(
            LocalEndpoint::parse("http://127.0.0.1:11434/")
                .unwrap()
                .display(),
            DEFAULT_OLLAMA_ENDPOINT
        );
        assert_eq!(
            LocalEndpoint::parse("http://[::1]:11434/")
                .unwrap()
                .display(),
            "http://[::1]:11434/"
        );
    }

    #[test]
    fn rejects_remote_and_ambiguous_origins() {
        for candidate in [
            "https://127.0.0.1:11434/",
            "http://example.com:11434/",
            "http://10.0.0.5:11434/",
            "http://127.0.0.1:11434/api/",
            "http://user@127.0.0.1:11434/",
            "http://127.0.0.1:11434/?endpoint=remote",
            "http://127.0.0.1:11434/#remote",
        ] {
            assert!(
                LocalEndpoint::parse(candidate).is_err(),
                "accepted {candidate}"
            );
        }
    }
}
