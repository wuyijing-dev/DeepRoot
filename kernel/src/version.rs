//! Build-time version stamp from repository `VERSION` (Linux-inspired).
//!
//! The VERSION file's first non-empty, non-`#` line is the current release;
//! remaining lines are the roadmap (comments).

const RAW: &str = include_str!("../../VERSION");

/*
 * version_string - return the current release from VERSION
 *
 * Skips blank lines and `#` roadmap comments so the banner stays short.
 */
pub fn version_string() -> &'static str {
    RAW.lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .unwrap_or("unknown")
}
