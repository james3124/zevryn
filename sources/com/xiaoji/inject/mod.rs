//! Decoder for the bundled `.xj` injection payloads (mirrored implementation).
//!
//! In the reference APK the three assets `apktool/assets/inject_{1,2,3}.xj`
//! are byte-reversed files. The Java runtime
//! (`com.xiaoji.inject.utils.InjectActivationUtils.n`) reverses the bytes
//! again and writes `xiaoji.bash`, `xjServer.jar` and `xjServer_a`.
//!
//! - `inject_1.xj` reverses to a UTF-8 `/system/bin/sh` script that copies
//!   `xjServer.jar` / `xjServer_a` into `/data/local/tmp/.xiaoji` and launches
//!   `com.xiaoji.inject.XiaojiInjectServer` through `app_process`.
//! - `inject_2.xj` and `inject_3.xj` reverse to a DEX and a ZIP respectively,
//!   so [`decode`] legitimately returns a UTF-8 error for those two.
//!
//! The decoder is dependency-free (std only) and mirrors the API of the core
//! crate's `src/inject.rs` so the two stay interchangeable.

/// A decoded `.xj` payload plus the shell variables extracted from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InjectPayload {
    pub script: String,
    pub device: Option<String>,
    pub model: Option<String>,
    pub version: Option<String>,
    pub package_placeholder: bool,
}

/// Reverse a byte-reversed payload and decode it as UTF-8.
pub fn decode(bytes: &[u8]) -> Result<String, std::string::FromUtf8Error> {
    let reversed: Vec<u8> = bytes.iter().rev().copied().collect();
    String::from_utf8(reversed)
}

impl InjectPayload {
    /// Decode a payload and extract `device="$1"`, `XJMODEL="$2"`,
    /// `XJVER="$3"` and the `PACKAGE="package_name"` placeholder.
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let script = decode(bytes).map_err(|e| e.to_string())?;
        Ok(Self {
            script: script.clone(),
            device: shell_var(&script, "device"),
            model: shell_var(&script, "XJMODEL"),
            version: shell_var(&script, "XJVER"),
            package_placeholder: script.contains("PACKAGE=\"package_name\""),
        })
    }
}

/// Extract a simple `NAME="value"` shell assignment.
fn shell_var(script: &str, name: &str) -> Option<String> {
    let prefix = format!("{name}=\"");
    let start = script.find(&prefix)? + prefix.len();
    let rest = &script[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const REFERENCE_ASSETS: &str =
        "/home/Project/Zevryn/reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/apktool/assets";
    const LOCAL_ASSETS: &str = "/home/Project/Zevryn/app/apktool/assets";

    /// Read a real asset from the absolute reference path, falling back to the
    /// mirrored local copy, and skipping the test if neither is present.
    fn read_asset(name: &str) -> Option<Vec<u8>> {
        std::fs::read(format!("{REFERENCE_ASSETS}/{name}"))
            .or_else(|_| std::fs::read(format!("{LOCAL_ASSETS}/{name}")))
            .ok()
    }

    #[test]
    fn decode_roundtrip() {
        let original = b"#!/system/bin/sh\necho hi\n";
        let reversed: Vec<u8> = original.iter().rev().copied().collect();
        assert_eq!(decode(&reversed).unwrap(), std::str::from_utf8(original).unwrap());
    }

    #[test]
    fn parse_payload() {
        let script = "#!/system/bin/sh\ndevice=\"$1\"\nXJMODEL=\"$2\"\nXJVER=\"$3\"\nPACKAGE=\"package_name\"\n";
        let rev: Vec<u8> = script.as_bytes().iter().rev().copied().collect();
        let p = InjectPayload::parse(&rev).unwrap();
        assert_eq!(p.device.as_deref(), Some("$1"));
        assert_eq!(p.model.as_deref(), Some("$2"));
        assert_eq!(p.version.as_deref(), Some("$3"));
        assert!(p.package_placeholder);
        assert_eq!(p.script, script);
    }

    #[test]
    fn missing_variables_are_none() {
        let script = "#!/system/bin/sh\necho hi\n";
        let rev: Vec<u8> = script.as_bytes().iter().rev().copied().collect();
        let p = InjectPayload::parse(&rev).unwrap();
        assert_eq!(p.device, None);
        assert_eq!(p.model, None);
        assert_eq!(p.version, None);
        assert!(!p.package_placeholder);
    }

    #[test]
    fn parses_real_payload_when_present() {
        let Some(bytes) = read_asset("inject_1.xj") else { return; };
        let p = InjectPayload::parse(&bytes).unwrap();
        assert!(p.script.starts_with("#!/system/bin/sh"));
        assert_eq!(p.device.as_deref(), Some("$1"));
        assert_eq!(p.model.as_deref(), Some("$2"));
        assert_eq!(p.version.as_deref(), Some("$3"));
        assert!(p.package_placeholder);
        assert!(p.script.contains("xjServer.jar"));
        assert!(p.script.contains("app_process"));
    }

    #[test]
    fn binary_payloads_fail_utf8_gracefully() {
        // inject_2 reverses to a DEX, inject_3 to a ZIP — neither is UTF-8.
        for name in ["inject_2.xj", "inject_3.xj"] {
            let Some(bytes) = read_asset(name) else { continue; };
            assert!(decode(&bytes).is_err(), "{name} should not decode as UTF-8");
            assert!(InjectPayload::parse(&bytes).is_err());
        }
    }
}
