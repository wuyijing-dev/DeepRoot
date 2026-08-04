//! Syscall numbers — not the Linux table; capability microkernel ops.

pub const SYS_DEBUG_WRITE: usize = 0;
pub const SYS_LEDGER_DUMP: usize = 1;
pub const SYS_CAP_DERIVE: usize = 2;
pub const SYS_IPC_CALL: usize = 3;
pub const SYS_CAP_REVOKE: usize = 4;
pub const SYS_CAP_MINT: usize = 5;
pub const SYS_IPC_RECV: usize = 6;
pub const SYS_IPC_REPLY: usize = 7;
pub const SYS_YIELD: usize = 8;
pub const SYS_EXIT: usize = 9;
