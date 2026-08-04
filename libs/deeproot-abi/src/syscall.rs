//! Syscall numbers and errno — ABI freeze candidate (0.8 / 1.0).
//!
//! Do not renumber after 1.0.0; additive syscalls only in 1.1+.
//!
//! Return convention in `a0`:
//! - `0` or positive — success (often a label, slot index, or byte count)
//! - `-1` — generic failure (`ERR_GENERIC`)
//! - `-11` — try again / would block without parking (`ERR_AGAIN`) — rare after 0.8.2
//! - `-38` — unknown syscall (`ERR_NOSYS`)

/// Success / generic failure.
pub const ERR_GENERIC: isize = -1;
/// Would block (legacy poll path).
pub const ERR_AGAIN: isize = -11;
/// Unknown syscall number.
pub const ERR_NOSYS: isize = -38;

pub const SYS_DEBUG_WRITE: usize = 0;
pub const SYS_LEDGER_DUMP: usize = 1;
/// `a0`=parent slot, `a1`=rights, `a2`=CapType as u64, `a3`=badge → new slot or err
pub const SYS_CAP_MINT: usize = 5;
/// `a0`=parent, `a1`=rights, `a2`=badge_mask → new slot or err
pub const SYS_CAP_DERIVE: usize = 2;
/// `a0`=slot → number of slots cleared or err
pub const SYS_CAP_REVOKE: usize = 4;
/// `a0`=ep slot, `a1`=label, `a2`=word0 → reply label (blocks until reply)
pub const SYS_IPC_CALL: usize = 3;
/// `a0`=endpoint badge → message label (blocks if empty)
pub const SYS_IPC_RECV: usize = 6;
/// `a0`=badge, `a1`=label, `a2`=word0 → 0; wakes blocked caller
pub const SYS_IPC_REPLY: usize = 7;
pub const SYS_YIELD: usize = 8;
pub const SYS_EXIT: usize = 9;
