//! In-kernel syscall helpers (kept for non-sched call sites).

use crate::cap::{TaskId, TaskTable};
use crate::ipc::EndpointTable;
use crate::sched;

/*
 * dispatch - forward to the scheduler syscall table (single source of truth)
 */
#[allow(dead_code)]
pub fn dispatch(
    tasks: &mut TaskTable,
    eps: &mut EndpointTable,
    _current: TaskId,
    nr: usize,
    a0: u64,
    a1: u64,
    a2: u64,
    a3: u64,
) -> isize {
    sched::handle_syscall(tasks, eps, nr, a0, a1, a2, a3)
}
