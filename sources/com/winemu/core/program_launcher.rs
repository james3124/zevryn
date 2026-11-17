//! Rust model of the portable parts of
//! `com.winemu.core.server.environment.plugins.ProgramLauncher` and the env
//! keys set by `com.winemu.core.controller.EnvironmentController`
//! (GameHub Lite v5.1.8).
//!
//! Two behaviors are portable and kept here:
//! 1. ELF header sniffing (`ProgramLauncher.h(File)`): x86 ELF binaries (e_machine
//!    3 or 62) require Box64 translation before launch.
//! 2. The base exec argv shape (`ProgramLauncher.e()`):
//!    `linker64 [box64] <cmd> [extra]`, with the environment variable names
//!    those controllers populate.
//!
//! Native process launch (`ProcessHelper.g`, `Process.killProcess`), PATH
//! lookup on the Android filesystem, `EnvVars`/`Callback` plumbing and the Wine
//! executable are out of scope. Kept name-for-name compatible with
//! `src/emu.rs`.

/// ELF machine types checked by `ProgramLauncher.h()` to decide whether Box64
/// translation is required.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfMachine {
    I386,   // 3  — 32-bit x86 (EM_386)
    X86_64, // 62 — 64-bit x86 (EM_X86_64)
    Other(u16),
}

impl ElfMachine {
    /// `i5 == 3 || i5 == 62` in `ProgramLauncher.h()`.
    pub fn needs_box64(self) -> bool {
        matches!(self, Self::I386 | Self::X86_64)
    }
}

/// Inspect the first 20 bytes of a candidate executable, exactly as the Java
/// `ProgramLauncher.h(File)` does: ELF magic, class 1/2, data 1/2, then e_machine
/// decoded little- or big-endian from bytes 18..20. Returns `None` for anything
/// that is not a readable ELF header with at least 20 bytes.
pub fn inspect_elf(header: &[u8]) -> Option<ElfMachine> {
    if header.len() < 20 {
        return None;
    }
    if &header[..4] != b"\x7fELF" {
        return None;
    }
    if !matches!(header[4], 1 | 2) || !matches!(header[5], 1 | 2) {
        return None;
    }
    let machine = if header[5] == 1 {
        u16::from_le_bytes([header[18], header[19]])
    } else {
        u16::from_be_bytes([header[18], header[19]])
    };
    Some(match machine {
        3 => ElfMachine::I386,
        62 => ElfMachine::X86_64,
        other => ElfMachine::Other(other),
    })
}

/// Environment variable names set by `EnvironmentController.b()/c()/...`.
/// Mirrors the reference set from `src/emu.rs`.
pub const ENV_KEYS: &[&str] = &[
    "WINEMU_ROOT_FS", "WINEMU_VFS", "PATH", "LD_PRELOAD", "LD_LIBRARY_PATH",
    "LC_ALL", "LC_CTYPE", "HOME", "USER", "DISPLAY", "TMPDIR", "XDG_RUNTIME_DIR",
    "BOX64_LOG", "BOX64_SHOWBT", "BOX64_RCFILE", "BOX64_DYNAREC", "BOX64_MMAP32",
    "DXVK_LOG_LEVEL", "DXVK_ASYNC", "DXVK_STATE_CACHE_PATH",
];

/// Base command shape used by `ProgramLauncher.e()`:
/// `arrayList.add("/system/bin/linker64")`, then (when box64 is needed) the
/// `box64` path, then the command, then any extra args.
pub fn exec_argv(
    command: &[String],
    extra: &[String],
    needs_box64: bool,
    box64_path: &str,
) -> Vec<String> {
    let mut argv = vec!["/system/bin/linker64".to_owned()];
    if needs_box64 {
        argv.push(box64_path.to_owned());
    }
    argv.extend_from_slice(command);
    argv.extend_from_slice(extra);
    argv
}

/// `ProgramLauncher` defaults: the linker prefix and the fallback box64 name
/// used when no custom path was configured (`str == null ? "box64" : str`).
pub const LINKER64: &str = "/system/bin/linker64";
pub const DEFAULT_BOX64: &str = "box64";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_x86_elf() {
        let mut h = [0u8; 20];
        h[..4].copy_from_slice(b"\x7fELF");
        h[4] = 2; // ELF64
        h[5] = 1; // little endian
        h[18] = 62; // EM_X86_64
        assert_eq!(inspect_elf(&h), Some(ElfMachine::X86_64));
        assert!(inspect_elf(&h).unwrap().needs_box64());

        h[18] = 3; // EM_386
        assert_eq!(inspect_elf(&h), Some(ElfMachine::I386));
        assert!(inspect_elf(&h).unwrap().needs_box64());

        h[18] = 183; // EM_AARCH64
        assert_eq!(inspect_elf(&h), Some(ElfMachine::Other(183)));
        assert!(!inspect_elf(&h).unwrap().needs_box64());
    }

    #[test]
    fn detects_big_endian_machine() {
        let mut h = [0u8; 20];
        h[..4].copy_from_slice(b"\x7fELF");
        h[4] = 1;
        h[5] = 2; // big endian
        h[18] = 0;
        h[19] = 62;
        assert_eq!(inspect_elf(&h), Some(ElfMachine::X86_64));
    }

    #[test]
    fn rejects_non_elf() {
        assert_eq!(inspect_elf(b"MZ\x90\x00"), None);
        assert_eq!(inspect_elf(&[0u8; 10]), None);
        let mut h = [0u8; 20];
        h[..4].copy_from_slice(b"\x7fELF");
        h[4] = 9; // invalid class
        h[5] = 1;
        assert_eq!(inspect_elf(&h), None);
        h[4] = 2;
        h[5] = 3; // invalid data encoding
        assert_eq!(inspect_elf(&h), None);
    }

    #[test]
    fn builds_exec_argv() {
        let cmd = vec!["wine".to_owned(), "game.exe".to_owned()];
        let argv = exec_argv(&cmd, &[], true, "/opt/bin/box64");
        assert_eq!(
            argv,
            vec!["/system/bin/linker64", "/opt/bin/box64", "wine", "game.exe"]
        );
        let no_box = exec_argv(&cmd, &["-v".into()], false, "");
        assert_eq!(no_box, vec![LINKER64, "wine", "game.exe", "-v"]);
    }

    #[test]
    fn env_keys_are_the_expected_set() {
        assert_eq!(ENV_KEYS.len(), 20);
        assert!(ENV_KEYS.contains(&"BOX64_DYNAREC"));
        assert!(ENV_KEYS.contains(&"DXVK_STATE_CACHE_PATH"));
    }
}
