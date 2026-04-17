//! PR #10 blocker fix: metrics server panics on port conflict.
//! `MetricsServer::bind_or_noop` catches `AddrInUse` (and any other bind
//! error) and returns a no-op handle. The process continues running without
//! metrics rather than unwinding with a fatal panic.

use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub enum MetricsHandle {
    Live { addr: SocketAddr },
    Noop { reason: String },
}

impl MetricsHandle {
    pub fn is_live(&self) -> bool {
        matches!(self, Self::Live { .. })
    }

    pub fn addr(&self) -> Option<SocketAddr> {
        match self {
            Self::Live { addr } => Some(*addr),
            Self::Noop { .. } => None,
        }
    }
}

pub struct MetricsServer;

impl MetricsServer {
    /// Attempt to bind a TCP listener on `addr`.
    /// Returns a `MetricsHandle::Live` on success.
    /// On any bind failure (including `AddrInUse`) logs a warning and returns
    /// a `MetricsHandle::Noop` – the caller must not panic on this path.
    pub fn bind_or_noop(addr: SocketAddr) -> MetricsHandle {
        match std::net::TcpListener::bind(addr) {
            Ok(listener) => {
                // Store the actual bound address in case the caller passed port 0
                let bound = listener
                    .local_addr()
                    .unwrap_or(addr);
                // Intentionally drop the listener here; production code would
                // pass it to a metrics server runtime (prometheus, axum, etc.)
                drop(listener);
                MetricsHandle::Live { addr: bound }
            }
            Err(e) => {
                eprintln!(
                    "[ghost] metrics server bind failed on {} – continuing without metrics: {}",
                    addr, e
                );
                MetricsHandle::Noop {
                    reason: format!("bind error on {}: {}", addr, e),
                }
            }
        }
    }
}
