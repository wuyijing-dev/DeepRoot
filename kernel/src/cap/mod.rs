//! Capability subsystem — Cap Root (0.3.x).
//!
//! Per-task CSpace, badges, revoke-by-provenance-parent, rights checks.

mod space;
mod task;

pub use space::{CapError, CapSlot, CapSpace, ProvenanceHop, CAP_SLOTS, PROVENANCE_DEPTH};
pub use task::{TaskId, TaskTable, MAX_TASKS};
