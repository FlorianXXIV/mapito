pub mod error;

/// Number of Bytes in MiB
pub const MIB: u64 = 0x100000;
/// Number of Bytes in KiB
pub const KIB: u64 = 0x400;

pub fn byte_to_readable(bytes: u64) -> String {
    let suffix;
    let size;
    if bytes < MIB {
        size = bytes as f32 / KIB as f32;
        suffix = "KiB";
    } else {
        size = bytes as f32 / MIB as f32;
        suffix = "MiB";
    }

    format!("{:.2} {}", size, suffix)
}
