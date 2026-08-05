//! Syscall numbers and errno — frozen base (1.0) + additive 1.1–1.12.
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
/// `a0`=path ptr, `a1`=len (0 = list cwd / root) — print directory listing
pub const SYS_FS_LIST: usize = 12;
/// `a0`=path ptr, `a1`=len — print file contents (cwd-relative OK)
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
/// `a0`=path, `a1`=plen, `a2`=data, `a3`=dlen — write/create VFS text file
pub const SYS_FS_WRITE: usize = 22;

/* ---- 1.9 directories / cwd ---- */

/// `a0`=path, `a1`=len — create directory
pub const SYS_FS_MKDIR: usize = 23;
/// `a0`=path, `a1`=len — remove empty directory
pub const SYS_FS_RMDIR: usize = 24;
/// `a0`=path, `a1`=len — remove VFS file (not embed/DRFS)
pub const SYS_FS_UNLINK: usize = 25;
/// `a0`=path, `a1`=len — set task cwd
pub const SYS_CHDIR: usize = 26;
/// `a0`=buf, `a1`=buflen → bytes written (absolute path)
pub const SYS_GETCWD: usize = 27;

/* ---- 1.10 loadable servers ---- */

/// `a0`=path, `a1`=plen, `a2`=badge — spawn ELF server + mint EP to caller → cap slot
pub const SYS_SPAWN_SERVER: usize = 28;
/// Print module registry to console
pub const SYS_MODULE_LIST: usize = 29;
/// `a0`=src, `a1`=slen, `a2`=dst, `a3`=dlen — copy file onto VFS path
pub const SYS_FS_CP: usize = 30;

/* ---- 1.11 durable FS ---- */

/// `a0`=path, `a1`=plen, `a2`=data, `a3`=dlen — append (root → DRFS)
pub const SYS_FS_APPEND: usize = 31;

/* ---- 1.11.1 file descriptors ---- */

/// `a0`=path, `a1`=plen, `a2`=flags → fd
pub const SYS_OPEN: usize = 32;
/// `a0`=fd
pub const SYS_CLOSE: usize = 33;
/// `a0`=fd, `a1`=buf, `a2`=len → bytes read
pub const SYS_FD_READ: usize = 34;
/// `a0`=fd, `a1`=buf, `a2`=len → bytes written
pub const SYS_FD_WRITE: usize = 35;
/// `a0`=fd, `a1`=offset (isize), `a2`=whence → new offset
pub const SYS_LSEEK: usize = 36;

/* ---- 1.12 practical lab ---- */

/// `a0`=milliseconds — sleep / yield until elapsed
pub const SYS_SLEEP_MS: usize = 37;
/// Dump current task CapSpace to console
pub const SYS_CAP_DUMP: usize = 38;

/// Pass as `SYS_TASK_STDOUT` a1 to restore console output.
pub const STDOUT_CONSOLE: usize = usize::MAX;

/// QEMU virt / ACLINT mtime frequency used by `SYS_TIME`
pub const TIME_HZ: u64 = 10_000_000;
