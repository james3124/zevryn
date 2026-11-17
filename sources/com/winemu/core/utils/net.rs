//! Pure helpers modeled on `com.winemu.core.utils.NetworkHelper.Companion`
//! (GameHub Lite v5.1.8).
//!
//! The Android `WifiManager`/`DhcpInfo` plumbing (`a()` gateway, `b()` IP,
//! `c()` prefix from `NetworkInterface`) needs a device and is out of scope.
//! The two static string conversions are pure and kept here:
//!
//! * `Companion.a(int)` — little-endian `int` to dotted IPv4 string.
//! * `Companion.b(int)` — prefix length to dotted netmask string.

/// `NetworkHelper.Companion.a(i3)` — decode a 32-bit int (as returned by
/// `WifiInfo.getIpAddress()` / `DhcpInfo.gateway`) into dotted decimal.
/// Octets are `(i & 255), ((i>>8)&255), ((i>>16)&255), ((i>>24)&255)`.
pub fn ipv4_from_int(addr: i32) -> String {
    format!(
        "{}.{}.{}.{}",
        addr & 0xFF,
        (addr >> 8) & 0xFF,
        (addr >> 16) & 0xFF,
        (addr >> 24) & 0xFF
    )
}

/// `NetworkHelper.Companion.b(i3)` — prefix length to netmask string.
/// Java maps only 8, 16 and 24; everything else returns `0.0.0.0`.
pub fn netmask_from_prefix(prefix: i32) -> &'static str {
    match prefix {
        8 => "255.0.0.0",
        16 => "255.255.0.0",
        24 => "255.255.255.0",
        _ => "0.0.0.0",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipv4_decoding() {
        // 192.168.1.10 stored little-endian
        let addr = 0x0A01_A8C0u32 as i32;
        assert_eq!(ipv4_from_int(addr), "192.168.1.10");
        assert_eq!(ipv4_from_int(0), "0.0.0.0");
        assert_eq!(ipv4_from_int(-1), "255.255.255.255");
    }

    #[test]
    fn netmask_from_prefix_length() {
        assert_eq!(netmask_from_prefix(8), "255.0.0.0");
        assert_eq!(netmask_from_prefix(16), "255.255.0.0");
        assert_eq!(netmask_from_prefix(24), "255.255.255.0");
        assert_eq!(netmask_from_prefix(0), "0.0.0.0");
        assert_eq!(netmask_from_prefix(20), "0.0.0.0");
    }
}
