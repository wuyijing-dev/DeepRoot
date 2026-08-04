//! Capability ABI types shared by kernel and (future) userspace.
//!
//! Slot *storage* lives in the kernel; this module is the stable layout
//! learners can print / serialize for worksheets (0.3.0).

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
    /// Capability node / CSpace reference (teaching placeholder).
    CNode = 3,
    /// Physical frame (teaching placeholder until pager servers).
    Frame = 4,
}

/// Reason codes recorded in capability provenance (teaching microscope).
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
