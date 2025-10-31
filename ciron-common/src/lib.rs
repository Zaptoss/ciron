pub mod config;
pub mod transport;

// Generated gRPC code
pub mod generated {
    #[path = ""]
    pub mod ciron_v1 {
        include!("generated/ciron.v1.rs");
    }
}

pub use config::{load_config, GlobalConfig, ProgramConfig, TransportConfig};
pub use transport::Transport;

pub use generated::ciron_v1::ciron_daemon_client::CironDaemonClient;
pub use generated::ciron_v1::ciron_daemon_server::{CironDaemon, CironDaemonServer};
pub use generated::ciron_v1::*;
