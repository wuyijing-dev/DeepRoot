//! Syscall numbers and errno — frozen base (1.0) + additive 1.1–1.4.
//!
//! 0..9 frozen at 1.0.0. New numbers are additive only.

pub const ERR_GENERIC: isize = -1;
pub const ERR_AGAIN: isize = -11;
pub const ERR_NOSYS: isize = -38;

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

/// `a0` = embedded blob id (0=hello) → child sched id
pub const SYS_SPAWN: usize = 10;
/// Blocking-ish read of one console byte; `-11` if none ready
pub const SYS_DEBUG_READ: usize = 11;
/// Print ramfs directory listing to console
pub const SYS_FS_LIST: usize = 12;
/// `a0`=path ptr, `a1`=len — print file contents
pub const SYS_FS_CAT: usize = 13;
