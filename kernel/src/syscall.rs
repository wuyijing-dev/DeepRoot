//! In-kernel syscall dispatch (also used when not going through sched).

use crate::cap::{TaskId, TaskTable};
use crate::ipc::{call_from_cap, EndpointTable};
use crate::ledger::LEDGER;
use deeproot_abi::syscall::*;
use deeproot_abi::IpcMessage;

pub fn dispatch(
    tasks: &mut TaskTable,
    eps: &mut EndpointTable,
    current: TaskId,
    nr: usize,
    a0: u64,
    a1: u64,
    a2: u64,
    _a3: u64,
) -> isize {
    match nr {
        SYS_LEDGER_DUMP => {
            LEDGER.dump_to_console();
            0
        }
        SYS_IPC_CALL => {
            let mut msg = IpcMessage::with_label(a1);
            msg.words[0] = a2;
            match call_from_cap(tasks, eps, current, a0 as usize, msg) {
                Ok(()) => 0,
                Err(_) => -1,
            }
        }
        SYS_IPC_RECV => {
            let badge = a0;
            let cs = match tasks.cspace_mut(current) {
                Some(c) => c,
                None => return -1,
            };
            match eps.recv(current, badge, cs) {
                Ok(m) => m.label as isize,
                Err(_) => -11, /* EAGAIN — caller should yield */
            }
        }
        SYS_IPC_REPLY => {
            let badge = a0;
            let mut msg = IpcMessage::with_label(a1);
            msg.words[0] = a2;
            match eps.reply(current, badge, msg) {
                Ok(()) => 0,
                Err(_) => -1,
            }
        }
        SYS_DEBUG_WRITE | SYS_CAP_DERIVE | SYS_CAP_REVOKE | SYS_CAP_MINT | SYS_YIELD | SYS_EXIT => {
            -38
        }
        _ => -38,
    }
}
