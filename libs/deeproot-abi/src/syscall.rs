//! Syscall numbers and errno — frozen base (1.0) + additive 1.1–1.8.
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
/// `a0`=path ptr, `a1`=len — load ELF from ramfs and spawn
pub const SYS_EXEC: usize = 14;
/// Monotonic milliseconds since boot (SBI `time` / timer Hz)
pub const SYS_TIME: usize = 15;
/// `a0` = sched id from spawn/exec; 0 if exited+reaped, `-11` if still running
pub const SYS_WAIT: usize = 16;

/* ---- 1.8 pipe / redirect / mutable text ---- */

/// Create a byte pipe → pipe id
pub const SYS_PIPE: usize = 17;
/// `a0`=pipe id — tear down
pub const SYS_PIPE_CLOSE: usize = 18;
/// `a0`=pipe, `a1`=buf, `a2`=len → bytes read (0 if empty)
pub const SYS_PIPE_READ: usize = 19;
/// `a0`=pipe, `a1`=buf, `a2`=len → bytes written
pub const SYS_PIPE_WRITE: usize = 20;
/// `a0`=sched id, `a1`=pipe id or `STDOUT_CONSOLE` — redirect DEBUG_WRITE
pub const SYS_TASK_STDOUT: usize = 21;
/// `a0`=path, `a1`=plen, `a2`=data, `a3`=dlen — write/create scratch text file
pub const SYS_FS_WRITE: usize = 22;

/// Pass as `SYS_TASK_STDOUT` a1 to restore console output.
pub const STDOUT_CONSOLE: usize = usize::MAX;

/// QEMU virt / ACLINT mtime frequency used by `SYS_TIME`
pub const TIME_HZ: u64 = 10_000_000;
