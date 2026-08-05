//! Teaching framebuffer geometry (1.15) — shared by kernel docs / userspace.

/// Width in pixels (QEMU ramfb cfg).
pub const FB_WIDTH: u32 = 320;
/// Height in pixels.
pub const FB_HEIGHT: u32 = 240;
/// Bytes per pixel (XRGB8888 / XR24).
pub const FB_BPP: u32 = 4;
/// Row stride in bytes.
pub const FB_STRIDE: u32 = FB_WIDTH * FB_BPP;
/// Total framebuffer bytes.
pub const FB_BYTES: usize = (FB_STRIDE * FB_HEIGHT) as usize;
/// Pages needed for the pixel buffer (rounded up).
pub const FB_PAGES: usize = (FB_BYTES + 4095) / 4096;

/// DRM fourcc little-endian spelling of XR24 (XRGB8888), host LE value
/// before `.to_be()` when writing RamFBCfg.
pub const FB_FOURCC_XR24: u32 =
    (b'X' as u32) | ((b'R' as u32) << 8) | ((b'2' as u32) << 16) | ((b'4' as u32) << 24);
