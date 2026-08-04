//! Capability rights bits — subsettable on derive.

pub mod bits {
    pub const READ: u32 = 1 << 0;
    pub const WRITE: u32 = 1 << 1;
    pub const GRANT: u32 = 1 << 2;
    pub const IPC: u32 = 1 << 3;
    pub const ALL: u32 = READ | WRITE | GRANT | IPC;
}

/// Re-export bits at module root for `deeproot_abi::rights::READ` style.
pub use bits::*;

/// Newtype for clearer call sites (optional sugar).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rights(pub u32);

impl Rights {
    pub const fn new(bits: u32) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> u32 {
        self.0
    }

    pub const fn contains(self, need: u32) -> bool {
        self.0 & need == need
    }

    pub const fn is_subset_of(self, parent: u32) -> bool {
        self.0 & !parent == 0
    }
}

/*
 * rights_name - tiny debug helper for UART dumps
 */
pub fn rights_name(r: u32) -> &'static str {
    match r {
        bits::ALL => "ALL",
        x if x == bits::READ | bits::IPC => "READ|IPC",
        x if x == bits::READ => "READ",
        x if x == bits::IPC => "IPC",
        x if x == bits::GRANT => "GRANT",
        _ => "mixed",
    }
}
