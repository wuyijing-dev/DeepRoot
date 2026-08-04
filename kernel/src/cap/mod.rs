//! Capability subsystem — Cap Root (0.3.x).
//!
//! Per-task CSpace, badges, revoke-by-provenance-parent, rights checks.

mod space;
mod task;

pub use space::{CapError, CapSpace};
pub use task::{TaskId, TaskTable};
