//! Capability ABI types shared by kernel and userspace.
//!
//! Slot storage lives in the kernel; this module is the stable wire/layout
//! for CapType, rights, and CapView.

/// Object type encoded in a capability slot.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapType {
    /// Empty slot.
    Null = 0,
    /// Untyped / generic authority (boot roots start here).
    Untyped = 1,
    /// IPC endpoint (filled in 0.4.x).
    Endpoint = 2,
    /// Capability node / CSpace reference.
    CNode = 3,
    /// Physical frame (pager / frame server ownership).
    Frame = 4,
    /// Device interrupt (badge = IRQ number; PLIC wait via SYS_IRQ_WAIT in 1.16).
    Irq = 5,
}

/// Reason codes recorded in capability provenance.
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapReason {
    BootRoot = 0,
    Derive = 1,
    Mint = 2,
    Revoke = 3,
    Badge = 4,
}

/// Userspace-visible snapshot of one slot (no kernel pointers).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CapView {
    pub live: u8,
    pub cap_type: u8,
    pub _pad: u16,
    pub rights: u32,
    pub badge: u64,
    pub parent: u16,
    pub reason: u16,
}

impl CapView {
    pub const fn empty() -> Self {
        Self {
            live: 0,
            cap_type: CapType::Null as u8,
            _pad: 0,
            rights: 0,
            badge: 0,
            parent: u16::MAX,
            reason: 0,
        }
    }
}
