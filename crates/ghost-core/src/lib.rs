pub mod db;
pub mod gateway;
pub mod orchestrator;
pub mod provider;
pub mod reporting;

pub struct GhostCore;

impl GhostCore {
    pub fn new() -> Self {
        Self
    }
}
