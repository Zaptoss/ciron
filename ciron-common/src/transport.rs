use anyhow::{Context, Result};
use std::{path::PathBuf, u32};

#[derive(Debug, Clone)]
pub enum Transport {
    Inet { host: String, port: u16 },
    Unix { path: PathBuf },    
    Vsock { cid: u32, port: u32 },
}

impl Transport {
    pub fn default_inet() -> Self {
        Transport::Inet {
            host: "127.0.0.1".to_string(),
            port: 50051,
        }
    }
    
    pub fn default_unix() -> Self {
        Transport::Unix {
            path: PathBuf::from("/tmp/cirond.sock"),
        }
    }
    
    pub fn default_vsock() -> Self {
        Transport::Vsock {
            cid: u32::MAX,
            port: 50051,
        }
    }
    
    /// Parse transport from string
    /// 
    /// Formats:
    /// - `inet://host:port` or `tcp://host:port`
    /// - `unix:///path/to/socket`
    /// - `vsock://cid:port`
    pub fn parse(s: &str) -> Result<Self> {
        if let Some(addr) = s.strip_prefix("inet://").or_else(|| s.strip_prefix("tcp://")) {
            let parts: Vec<&str> = addr.split(':').collect();
            if parts.len() != 2 {
                anyhow::bail!("Invalid inet address format. Expected host:port");
            }
            let host = parts[0].to_string();
            let port = parts[1].parse().context("Invalid port number")?;
            Ok(Transport::Inet { host, port })
        } else if let Some(path) = s.strip_prefix("unix://") {
            Ok(Transport::Unix {
                path: PathBuf::from(path),
            })
        } else if let Some(addr) = s.strip_prefix("vsock://") {
            let parts: Vec<&str> = addr.split(':').collect();
            if parts.len() != 2 {
                anyhow::bail!("Invalid vsock address format. Expected cid:port");
            }
            let cid = parts[0].parse().context("Invalid CID")?;
            let port = parts[1].parse().context("Invalid port")?;
            Ok(Transport::Vsock { cid, port })
        } else {
            anyhow::bail!(
                "Unknown transport type. Use inet://, unix://, or vsock:// prefix"
            )
        }
    }
}

impl std::fmt::Display for Transport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Transport::Inet { host, port } => write!(f, "inet://{}:{}", host, port),
            Transport::Unix { path } => write!(f, "unix://{}", path.display()),
            Transport::Vsock { cid, port } => write!(f, "vsock://{}:{}", cid, port),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_inet() {
        let transport = Transport::parse("inet://127.0.0.1:8080").unwrap();
        match transport {
            Transport::Inet { host, port } => {
                assert_eq!(host, "127.0.0.1");
                assert_eq!(port, 8080);
            }
            _ => panic!("Expected Inet transport"),
        }
    }

    #[test]
    fn test_parse_unix() {
        let transport = Transport::parse("unix:///tmp/test.sock").unwrap();
        match transport {
            Transport::Unix { path } => {
                assert_eq!(path, PathBuf::from("/tmp/test.sock"));
            }
            _ => panic!("Expected Unix transport"),
        }
    }

    #[test]
    fn test_parse_vsock() {
        let transport = Transport::parse("vsock://2:9000").unwrap();
        match transport {
            Transport::Vsock { cid, port } => {
                assert_eq!(cid, 2);
                assert_eq!(port, 9000);
            }
            _ => panic!("Expected Vsock transport"),
        }
    }
}
