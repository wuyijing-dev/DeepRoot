//! Shared ABI between DeepRoot kernel and userspace.
//!
//! **ABI freeze candidate (0.8.x → 1.0.0):** syscall numbers, `IpcMessage`,
//! `CapView` / `CapType` / rights, and ledger event layouts are stable.
//! Breaking changes require MAJOR bump after 1.0.0; 1.1+ is additive only.
//!
//! Keep this crate `no_std` and dependency-light so both sides can link it.

#![no_std]

pub mod cap;
pub mod ipc;
pub mod rights;
pub mod syscall;

pub use cap::{CapReason, CapType, CapView};
pub use ipc::{IpcMessage, IPC_WORDS};
pub use rights::{rights_name, Rights};

/*
 * Compact event kinds stored in the Root Ledger.
 */
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LedgerKind {
    Boot = 0,
    Trap = 1,
    CapDerive = 2,
    IpcSend = 3,
    IpcRecv = 4,
    Panic = 5,
    CapRevoke = 6,
    CapMint = 7,
}

/// One ledger record — fixed size for easy dumping over UART.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LedgerEvent {
    pub kind: u8,
    pub _pad: [u8; 3],
    pub a0: u32,
    pub a1: u32,
    pub a2: u32,
}

impl LedgerEvent {
    pub const fn new(kind: LedgerKind, a0: u32, a1: u32, a2: u32) -> Self {
        Self {
            kind: kind as u8,
            _pad: [0; 3],
            a0,
            a1,
            a2,
        }
    }
}
