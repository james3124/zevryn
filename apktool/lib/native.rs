//! Inventory of the native shared objects shipped by GameHub Lite v5.1.8.
//!
//! The reference APK bundles a single ABI (`arm64-v8a`) with 19 `.so` files.
//! These binaries are third-party / native artifacts. They are **not
//! reimplemented** here — this module only *inventories* them: name, byte size,
//! a one-line role and a disposition describing how a pure-Rust rewrite would
//! treat each one. See `REPORT.md` for the rationale and crate replacements.

/// Native ABIs present in the reference APK. Only one is shipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeAbi {
    /// 64-bit ARM, the only ABI in the reference build.
    Arm64V8a,
}

impl NativeAbi {
    /// The complete set of shipped ABIs (exactly one).
    pub const ALL: [NativeAbi; 1] = [NativeAbi::Arm64V8a];

    /// Android ABI folder / triplet name.
    pub const fn name(self) -> &'static str {
        match self {
            NativeAbi::Arm64V8a => "arm64-v8a",
        }
    }
}

impl std::fmt::Display for NativeAbi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// How a pure-Rust rewrite would handle a given native library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeDisposition {
    /// Must be kept and called through FFI (proprietary/protocol/emulator code).
    FfiRequired,
    /// Can be replaced by an equivalent Rust crate.
    CrateReplaceable,
    /// Optional hardware/feature specific blob that can be dropped.
    Droppable,
}

impl NativeDisposition {
    /// Stable string label.
    pub const fn as_str(self) -> &'static str {
        match self {
            NativeDisposition::FfiRequired => "FFI-required",
            NativeDisposition::CrateReplaceable => "crate-replaceable",
            NativeDisposition::Droppable => "droppable",
        }
    }
}

impl std::fmt::Display for NativeDisposition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One inventoried native library.
#[derive(Debug, Clone, Copy)]
pub struct NativeLibrary {
    /// File name as shipped under `lib/arm64-v8a/`.
    pub name: &'static str,
    /// Size of the `.so` file in bytes, taken from the reference APK.
    pub size_bytes: u64,
    /// One-line description of the library's role.
    pub role: &'static str,
    /// Suggested treatment in a pure-Rust rewrite.
    pub disposition: NativeDisposition,
}

/// The 19 shared objects in `lib/arm64-v8a/`, sorted by file name.
pub const NATIVE_LIBS: [NativeLibrary; 19] = [
    NativeLibrary {
        name: "libGamesir.so",
        size_bytes: 27_232,
        role: "GameSir gamepad controller SDK JNI bridge.",
        disposition: NativeDisposition::Droppable,
    },
    NativeLibrary {
        name: "libadb.so",
        size_bytes: 550_928,
        role: "ADB (Android Debug Bridge) client used to install/debug remote devices.",
        disposition: NativeDisposition::FfiRequired,
    },
    NativeLibrary {
        name: "libavutil-55.so",
        size_bytes: 366_152,
        role: "FFmpeg AVUtil core (FFmpeg 3.x ABI 55) supporting media playback.",
        disposition: NativeDisposition::FfiRequired,
    },
    NativeLibrary {
        name: "libblur-lib.so",
        size_bytes: 9_944,
        role: "Native image blur filter invoked through JNI.",
        disposition: NativeDisposition::CrateReplaceable,
    },
    NativeLibrary {
        name: "libgpuinfo.so",
        size_bytes: 340_264,
        role: "GPU / OpenGL ES device information and capability probe.",
        disposition: NativeDisposition::CrateReplaceable,
    },
    NativeLibrary {
        name: "libijkplayer.so",
        size_bytes: 545_704,
        role: "IjkPlayer (FFmpeg-based) video playback engine.",
        disposition: NativeDisposition::FfiRequired,
    },
    NativeLibrary {
        name: "libijksdl.so",
        size_bytes: 362_832,
        role: "IjkPlayer SDL-compatible audio/video output layer.",
        disposition: NativeDisposition::FfiRequired,
    },
    NativeLibrary {
        name: "libimage_processing_util_jni.so",
        size_bytes: 28_944,
        role: "General image-processing utility routines exposed over JNI.",
        disposition: NativeDisposition::CrateReplaceable,
    },
    NativeLibrary {
        name: "libimagescale.so",
        size_bytes: 55_280,
        role: "Fast native image scaling / resampling helper.",
        disposition: NativeDisposition::CrateReplaceable,
    },
    NativeLibrary {
        name: "libjl_ota_auth.so",
        size_bytes: 15_704,
        role: "Jieli (JL) vendor OTA firmware authentication helper.",
        disposition: NativeDisposition::Droppable,
    },
    NativeLibrary {
        name: "liblog.so",
        size_bytes: 66_376,
        role: "Native Android log buffer read/write shim.",
        disposition: NativeDisposition::CrateReplaceable,
    },
    NativeLibrary {
        name: "libmmkv.so",
        size_bytes: 586_640,
        role: "Tencent MMKV high-performance key-value storage engine.",
        disposition: NativeDisposition::CrateReplaceable,
    },
    NativeLibrary {
        name: "libswscale-4.so",
        size_bytes: 349_736,
        role: "FFmpeg software pixel-format conversion / scaling (SWSCALE 4).",
        disposition: NativeDisposition::FfiRequired,
    },
    NativeLibrary {
        name: "libvfs.so",
        size_bytes: 984_680,
        role: "Virtual file-system layer for streaming/mounting game archives.",
        disposition: NativeDisposition::FfiRequired,
    },
    NativeLibrary {
        name: "libwebdav_jni.so",
        size_bytes: 2_736_848,
        role: "WebDAV remote-storage client exposed over JNI.",
        disposition: NativeDisposition::CrateReplaceable,
    },
    NativeLibrary {
        name: "libwinemu.so",
        size_bytes: 592_856,
        role: "Windows-game emulation / compatibility glue for ARM translation.",
        disposition: NativeDisposition::FfiRequired,
    },
    NativeLibrary {
        name: "libxjps-jni.so",
        size_bytes: 3_185_896,
        role: "XJPS image/codec processing JNI bridge.",
        disposition: NativeDisposition::FfiRequired,
    },
    NativeLibrary {
        name: "libxserver.so",
        size_bytes: 3_913_824,
        role: "Embedded X11 server powering the Windows-game compatibility layer.",
        disposition: NativeDisposition::FfiRequired,
    },
    NativeLibrary {
        name: "libzstd-jni-1.5.5-11.so",
        size_bytes: 595_880,
        role: "zstd-jni 1.5.5-11 native Zstandard compression binding.",
        disposition: NativeDisposition::CrateReplaceable,
    },
];

/// Sum of all inventoried native library sizes, in bytes.
pub const NATIVE_TOTAL_BYTES: u64 = {
    let mut total: u64 = 0;
    let mut i = 0;
    while i < NATIVE_LIBS.len() {
        total += NATIVE_LIBS[i].size_bytes;
        i += 1;
    }
    total
};

/// Look up a library by its file name.
pub fn find_library(name: &str) -> Option<&'static NativeLibrary> {
    NATIVE_LIBS.iter().find(|lib| lib.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_all_nineteen_libraries() {
        assert_eq!(NATIVE_LIBS.len(), 19);
    }

    #[test]
    fn only_arm64_abi_is_shipped() {
        assert_eq!(NativeAbi::ALL.len(), 1);
        assert_eq!(NativeAbi::ALL[0], NativeAbi::Arm64V8a);
        assert_eq!(NativeAbi::Arm64V8a.name(), "arm64-v8a");
        assert_eq!(NativeAbi::Arm64V8a.to_string(), "arm64-v8a");
    }

    #[test]
    fn library_names_are_unique_and_nonempty() {
        let mut names = std::collections::BTreeSet::new();
        for lib in NATIVE_LIBS.iter() {
            assert!(lib.name.ends_with(".so"), "{}", lib.name);
            assert!(names.insert(lib.name), "duplicate: {}", lib.name);
        }
        assert_eq!(names.len(), 19);
    }

    #[test]
    fn every_library_is_inventoried_with_a_role_and_size() {
        for lib in NATIVE_LIBS.iter() {
            assert!(lib.size_bytes > 0, "{} has no size", lib.name);
            assert!(!lib.role.is_empty(), "{} has no role", lib.name);
            assert!(!lib.disposition.as_str().is_empty());
        }
    }

    #[test]
    fn total_size_is_sum_of_parts() {
        let sum: u64 = NATIVE_LIBS.iter().map(|lib| lib.size_bytes).sum();
        assert_eq!(sum, NATIVE_TOTAL_BYTES);
        assert!(NATIVE_TOTAL_BYTES > 0);
    }

    #[test]
    fn lookup_finds_known_and_misses_unknown() {
        let zstd = find_library("libzstd-jni-1.5.5-11.so").expect("zstd present");
        assert_eq!(zstd.disposition, NativeDisposition::CrateReplaceable);
        assert!(find_library("libdoesnotexist.so").is_none());
    }

    #[test]
    fn dispositions_are_all_covered() {
        let count = |d: NativeDisposition| {
            NATIVE_LIBS
                .iter()
                .filter(|lib| lib.disposition == d)
                .count()
        };
        assert!(count(NativeDisposition::FfiRequired) > 0);
        assert!(count(NativeDisposition::CrateReplaceable) > 0);
        assert!(count(NativeDisposition::Droppable) > 0);
        assert_eq!(
            count(NativeDisposition::FfiRequired)
                + count(NativeDisposition::CrateReplaceable)
                + count(NativeDisposition::Droppable),
            19
        );
    }
}
