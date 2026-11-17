//! Portable firmware check / version comparison logic.
//!
//! Sources:
//! * `com.xiaoji.sdk.utils.VerCompareUtil.a(String,String)` — dot-separated
//!   integer comparison used across the OTA checks.
//! * `com.xj.ota.check.DefaultUpdateCheck` — `b()` (single version) and `c()`
//!   (manifest + device firmware list) update decisions.
//! * `com.xj.ota.check.G8UpdateCheck` — G8 uses `major-minor` pairs.
//! * `com.xj.ota.check.FirmwareUpdateCheckUtil` — strategy selection by
//!   device name / firmware version.
//!
//! No Android `Context`, threads or callbacks are represented.

use super::info::{FirmwareDataEntry, FirmwareEntry};
use std::cmp::Ordering;

/// Compare two dot-separated version strings.
///
/// This is a faithful, panic-free port of `VerCompareUtil.a`, returned as an
/// [`Ordering`] instead of `-1/0/1`. Non-numeric or missing segments are
/// treated as `0`, matching the "trailing positive segment" logic of the Java
/// method without throwing a `NumberFormatException`.
pub fn compare_versions(left: &str, right: &str) -> Ordering {
    if left == right {
        return Ordering::Equal;
    }
    let a: Vec<i64> = parse_segments(left);
    let b: Vec<i64> = parse_segments(right);
    let common = a.len().min(b.len());
    for i in 0..common {
        match a[i].cmp(&b[i]) {
            Ordering::Equal => continue,
            ord => return ord,
        }
    }
    // All shared segments equal: the side with a remaining non-zero segment wins.
    let a_tail_positive = a[common..].iter().any(|v| *v > 0);
    let b_tail_positive = b[common..].iter().any(|v| *v > 0);
    match (a_tail_positive, b_tail_positive) {
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        _ => Ordering::Equal,
    }
}

fn parse_segments(version: &str) -> Vec<i64> {
    version
        .split('.')
        .map(|s| s.trim().parse::<i64>().unwrap_or(0))
        .collect()
}

/// `VerCompareUtil.a` as the raw `{-1, 0, 1}` integer used by the Java callers.
pub fn compare_versions_i32(left: &str, right: &str) -> i32 {
    match compare_versions(left, right) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

/// Strategy selected by `FirmwareUpdateCheckUtil.a(String deviceName,
/// String firmwareVersion)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UpdateCheckStrategy {
    Default,
    G8,
    X3TypeC,
    X2TypeC,
    X2ProXbox,
}

impl UpdateCheckStrategy {
    pub fn for_device(device_name: &str, firmware_version: &str) -> Self {
        match device_name {
            "Gamesir-X3 Type-C" => UpdateCheckStrategy::X3TypeC,
            "Gamesir-G8" => UpdateCheckStrategy::G8,
            "GameSir-X2 Pro-Xbox" => {
                let v = compare_versions_i32(firmware_version, "128.0");
                let v2 = compare_versions_i32(firmware_version, "144.0");
                if v < 0 || v2 >= 0 {
                    UpdateCheckStrategy::Default
                } else {
                    UpdateCheckStrategy::X2ProXbox
                }
            }
            "GameSir-X2 Type-C" => UpdateCheckStrategy::X2TypeC,
            _ => UpdateCheckStrategy::Default,
        }
    }
}

/// `DefaultUpdateCheck.b(target, current, allow_different)`:
/// * `allow_different == true`  -> upgrade when versions differ (`!= 0`);
/// * `allow_different == false` -> upgrade only when target is newer (`> 0`).
pub fn needs_upgrade(target: &str, current: &str, allow_different: bool) -> bool {
    if target.is_empty() || current.is_empty() {
        return false;
    }
    let ord = compare_versions(target, current);
    if allow_different {
        ord != Ordering::Equal
    } else {
        ord == Ordering::Greater
    }
}

/// Strategy-aware version check. G8 compares `major-minor` pairs exactly as
/// `G8UpdateCheck`; other devices use [`needs_upgrade`].
pub fn needs_upgrade_with(
    strategy: UpdateCheckStrategy,
    target: &str,
    current: &str,
    allow_different: bool,
) -> bool {
    if strategy == UpdateCheckStrategy::G8 {
        return g8_needs_upgrade(target, current);
    }
    needs_upgrade(target, current, allow_different)
}

fn g8_needs_upgrade(target: &str, current: &str) -> bool {
    if target.is_empty() || current.is_empty() {
        return false;
    }
    let t: Vec<&str> = target.splitn(2, '-').collect();
    let c: Vec<&str> = current.splitn(2, '-').collect();
    if t.len() == 2 && c.len() == 2 {
        compare_versions(t[0], c[0]) != Ordering::Equal
            || compare_versions(t[1], c[1]) != Ordering::Equal
    } else {
        false
    }
}

/// `DefaultUpdateCheck.c(deviceFirmwareList, targetManifest, allowDifferent)`.
///
/// Only entries with `support_ota == true` are considered; a component is
/// upgradable when the target version differs (or is newer, depending on the
/// flag) from the device's current version.
pub fn is_update_available(
    device_firmware: &[FirmwareDataEntry],
    manifest: &[FirmwareEntry],
    allow_different: bool,
) -> bool {
    device_firmware
        .iter()
        .filter(|fw| fw.support_ota)
        .any(|fw| {
            manifest
                .iter()
                .find(|target| target.matches_type(&fw.firmware_type))
                .is_some_and(|target| needs_upgrade(&target.version, &fw.version, allow_different))
        })
}

/// `com.xj.ota.check.firmware.DefaultFirmwareCheck`: validate a firmware
/// archive path (must be a non-empty `.zip` whose name contains the vendor
/// prefix) and return the embedded version substring.
pub fn version_from_archive_path(path: &str, vendor_prefix: &str) -> Option<String> {
    let lower = path.to_ascii_lowercase();
    if !lower.ends_with(".zip") || !lower.contains(&vendor_prefix.to_ascii_lowercase()) {
        return None;
    }
    let start = lower.find(&vendor_prefix.to_ascii_lowercase())? + vendor_prefix.len();
    let end = lower.rfind(".zip")?;
    if start > end {
        return None;
    }
    Some(path[start..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_comparison_follows_ver_compare_util() {
        assert_eq!(compare_versions_i32("1.0", "1.0"), 0);
        assert_eq!(compare_versions_i32("1.1", "1.0"), 1);
        assert_eq!(compare_versions_i32("0.9", "1.0"), -1);
        assert_eq!(compare_versions_i32("128.0", "144.0"), -1);
        assert_eq!(compare_versions_i32("1.0.1", "1.0"), 1);
        assert_eq!(compare_versions_i32("1.0", "1.0.1"), -1);
        assert_eq!(compare_versions_i32("1.0", "1.0.0"), 0);
    }

    #[test]
    fn version_comparison_is_panic_free() {
        assert_eq!(compare_versions_i32("", "1.0"), -1);
        assert_eq!(compare_versions_i32("x.y", "1.0"), -1);
        assert_eq!(compare_versions_i32("1.beta", "1.0"), 0);
    }

    #[test]
    fn needs_upgrade_respects_allow_different() {
        assert!(needs_upgrade("1.1", "1.0", false));
        assert!(!needs_upgrade("1.0", "1.1", false));
        assert!(needs_upgrade("1.0", "1.1", true), "different is enough");
        assert!(!needs_upgrade("1.0", "1.0", true));
        assert!(!needs_upgrade("", "1.0", true));
    }

    #[test]
    fn strategy_selection_matches_firmware_update_check_util() {
        assert_eq!(
            UpdateCheckStrategy::for_device("Gamesir-G8", "1.0"),
            UpdateCheckStrategy::G8
        );
        assert_eq!(
            UpdateCheckStrategy::for_device("Gamesir-X3 Type-C", "1.0"),
            UpdateCheckStrategy::X3TypeC
        );
        assert_eq!(
            UpdateCheckStrategy::for_device("GameSir-X2 Type-C", "1.0"),
            UpdateCheckStrategy::X2TypeC
        );
        assert_eq!(
            UpdateCheckStrategy::for_device("GameSir-X2 Pro-Xbox", "130.0"),
            UpdateCheckStrategy::X2ProXbox
        );
        assert_eq!(
            UpdateCheckStrategy::for_device("GameSir-X2 Pro-Xbox", "150.0"),
            UpdateCheckStrategy::Default
        );
        assert_eq!(
            UpdateCheckStrategy::for_device("Other", "1.0"),
            UpdateCheckStrategy::Default
        );
    }

    #[test]
    fn g8_check_compares_major_minor_pairs() {
        assert!(!needs_upgrade_with(UpdateCheckStrategy::G8, "1.0-2.0", "1.0-2.0", true));
        assert!(needs_upgrade_with(UpdateCheckStrategy::G8, "1.0-2.1", "1.0-2.0", true));
        assert!(!needs_upgrade_with(UpdateCheckStrategy::G8, "1.0", "1.1", true));
    }

    #[test]
    fn manifest_check_skips_unsupported_components() {
        let device = vec![
            FirmwareDataEntry::new("main", "1.0", true),
            FirmwareDataEntry::new("pd", "2.0", false),
        ];
        let manifest = vec![
            FirmwareEntry::new("main", "1.0", "main.bin"),
            FirmwareEntry::new("pd", "9.9", "pd.bin"),
        ];
        assert!(!is_update_available(&device, &manifest, false));
        assert!(!is_update_available(&device, &manifest, true));

        let newer = vec![FirmwareEntry::new("main", "1.1", "main.bin")];
        assert!(is_update_available(&device, &newer, false));

        let downgrade = vec![
            FirmwareEntry::new("main", "0.9", "main.bin"),
        ];
        assert!(!is_update_available(&device, &downgrade, false));
        assert!(is_update_available(&device, &downgrade, true), "diff allowed");
    }

    #[test]
    fn version_from_archive_path_validates_prefix_and_extension() {
        assert_eq!(
            version_from_archive_path("/cache/X5s-firmware-1.2.3.zip", "X5s-firmware-").as_deref(),
            Some("1.2.3")
        );
        assert_eq!(version_from_archive_path("/cache/other.zip", "X5s-firmware-"), None);
        assert_eq!(version_from_archive_path("/cache/X5s-firmware-1.2.3.bin", "X5s-firmware-"), None);
    }
}
