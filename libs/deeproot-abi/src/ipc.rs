//! IPC message / endpoint ABI (0.4.x Ledger Vein).

/// Max fixed words in a teaching message (keep small for UART dumps).
pub const IPC_WORDS: usize = 4;

/// Fixed-size IPC payload shared by kernel and (future) userspace.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IpcMessage {
    pub label: u64,
    pub words: [u64; IPC_WORDS],
    /// Non-zero when a capability is transferred with the message.
    pub transfer_valid: u8,
    pub transfer_type: u8,
    pub _pad: u16,
    pub transfer_rights: u32,
    pub transfer_badge: u64,
}

impl IpcMessage {
    pub const fn empty() -> Self {
        Self {
            label: 0,
            words: [0; IPC_WORDS],
            transfer_valid: 0,
            transfer_type: 0,
            _pad: 0,
            transfer_rights: 0,
            transfer_badge: 0,
        }
    }

    pub const fn with_label(label: u64) -> Self {
        let mut m = Self::empty();
        m.label = label;
        m
    }
}
