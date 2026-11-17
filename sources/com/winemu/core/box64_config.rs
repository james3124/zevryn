//! Rust model of the portable part of `com.winemu.core.trans_layer.Box64Config`
//! (GameHub Lite v5.1.8).
//!
//! All 28 fields are plain integers/strings. The two pure data transforms
//! (`setCompatibilityMode()` / `setPerformanceMode()`) and the `export(EnvVars)`
//! mapping to `BOX64_*` environment keys are fully portable and modeled here
//! as a `Vec<(String, String)>`; applying them to a real process environment is
//! out of scope. `Parcelable`/`CREATOR` (Android IPC) is excluded.

/// All tunable knobs of Box64, as a plain data struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Box64Config {
    pub box64_path: String,
    pub dynarec: i32,
    pub safe_flags: i32,
    pub fast_nan: i32,
    pub fast_round: i32,
    pub x87_double: i32,
    pub big_block: i32,
    pub strong_mem: i32,
    pub forward: i32,
    pub callret: i32,
    pub wait: i32,
    pub aligned_atomics: i32,
    pub weak_barrier: i32,
    pub pause: i32,
    pub df: i32,
    pub dirty: i32,
    pub native_flags: i32,
    pub volatile_metadata: i32,
    pub div0: i32,
    pub dyna_cache: i32,
    pub dyna_cache_folder: String,
    pub dyna_cache_min: i32,
    pub mmap32: i32,
    pub rdtsc1_ghz: i32,
    pub cpu_type: i32,
    pub box64_avx: i32,
    pub ignore_int3: i32,
    pub cpu_name: String,
}

impl Default for Box64Config {
    /// Defaults from the synthetic constructor / no-arg constructor mask
    /// `i28 = 0x0FFFFFFF` (every default bit set).
    fn default() -> Self {
        Self {
            box64_path: String::new(),
            dynarec: 1,
            safe_flags: 1,
            fast_nan: 1,
            fast_round: 1,
            x87_double: 0,
            big_block: 1,
            strong_mem: 0,
            forward: 1024,
            callret: 1,
            wait: 0,
            aligned_atomics: 0,
            weak_barrier: 1,
            pause: 0,
            df: 1,
            dirty: 0,
            native_flags: 1,
            volatile_metadata: 1,
            div0: 0,
            dyna_cache: 2,
            dyna_cache_folder: String::new(),
            dyna_cache_min: 350,
            mmap32: 1,
            rdtsc1_ghz: 0,
            cpu_type: 0,
            box64_avx: 0,
            ignore_int3: 0,
            cpu_name: "GameFusion CPU".to_owned(),
        }
    }
}

impl Box64Config {
    /// Mirrors `setCompatibilityMode()`.
    pub fn set_compatibility_mode(&mut self) {
        self.safe_flags = 2;
        self.fast_nan = 0;
        self.fast_round = 0;
        self.x87_double = 1;
        self.big_block = 0;
        self.strong_mem = 1;
        self.wait = 0;
        self.callret = 0;
    }

    /// Mirrors `setPerformanceMode()`.
    pub fn set_performance_mode(&mut self) {
        self.safe_flags = 0;
        self.fast_nan = 1;
        self.fast_round = 1;
        self.x87_double = 0;
        self.big_block = 2;
        self.strong_mem = 0;
        self.forward = 512;
        self.wait = 1;
    }

    /// Mirrors `export(EnvVars)`: emit the ordered list of `BOX64_*` assignments.
    /// `BOX64_DYNAREC` is emitted twice (once as a fixed `1`, once as the
    /// configured value) exactly as in the reference; a map applies the last.
    pub fn export(&self) -> Vec<(String, String)> {
        fn push(out: &mut Vec<(String, String)>, key: &str, value: impl ToString) {
            out.push((key.to_owned(), value.to_string()));
        }

        let mut out = Vec::new();
        push(&mut out, "BOX64_NOBANNER", "1");
        push(&mut out, "BOX64_DYNAREC", "1");
        push(&mut out, "BOX64_X11GLX", "1");
        push(&mut out, "BOX64_NORCFILES", "1");
        push(&mut out, "BOX64_MMAP32", "0");
        push(&mut out, "BOX64_DYNAREC", self.dynarec);
        push(&mut out, "BOX64_DYNAREC_SAFEFLAGS", self.safe_flags);
        push(&mut out, "BOX64_DYNAREC_FASTNAN", self.fast_nan);
        push(&mut out, "BOX64_DYNAREC_FASTROUND", self.fast_round);
        push(&mut out, "BOX64_DYNAREC_X87DOUBLE", self.x87_double);
        push(&mut out, "BOX64_DYNAREC_BIGBLOCK", self.big_block);
        push(&mut out, "BOX64_DYNAREC_STRONGMEM", self.strong_mem);
        push(&mut out, "BOX64_DYNAREC_FORWARD", self.forward);
        push(&mut out, "BOX64_DYNAREC_CALLRET", self.callret);
        push(&mut out, "BOX64_DYNAREC_WAIT", self.wait);
        push(&mut out, "BOX64_DYNAREC_ALIGNED_ATOMICS", self.aligned_atomics);
        push(&mut out, "BOX64_DYNAREC_WEAKBARRIER", self.weak_barrier);
        push(&mut out, "BOX64_DYNAREC_PAUSE", self.pause);
        push(&mut out, "BOX64_DYNAREC_DF", self.df);
        push(&mut out, "BOX64_DYNAREC_DIRTY", self.dirty);
        push(&mut out, "BOX64_DYNAREC_NATIVEFLAGS", self.native_flags);
        push(&mut out, "BOX64_DYNAREC_VOLATILE_METADATA", self.volatile_metadata);
        push(&mut out, "BOX64_DYNAREC_DIV0", self.div0);
        push(&mut out, "BOX64_DYNACACHE", self.dyna_cache);
        if !self.dyna_cache_folder.is_empty() {
            push(&mut out, "BOX64_DYNACACHE_FOLDER", self.dyna_cache_folder.clone());
        }
        push(&mut out, "BOX64_DYNACACHE_MIN", self.dyna_cache_min);
        push(&mut out, "BOX64_MMAP32", self.mmap32);
        push(&mut out, "BOX64_RDTSC_1GHZ", self.rdtsc1_ghz);
        push(&mut out, "BOX64_CPUTYPE", self.cpu_type);
        push(&mut out, "BOX64_AVX", self.box64_avx);
        push(&mut out, "BOX64_IGNOREINT3", self.ignore_int3);
        push(&mut out, "BOX64_CPUNAME", self.cpu_name.clone());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_reference() {
        let c = Box64Config::default();
        assert_eq!(c.box64_path, "");
        assert_eq!(c.dynarec, 1);
        assert_eq!(c.forward, 1024);
        assert_eq!(c.dyna_cache, 2);
        assert_eq!(c.dyna_cache_min, 350);
        assert_eq!(c.cpu_name, "GameFusion CPU");
    }

    #[test]
    fn compatibility_and_performance_modes() {
        let mut c = Box64Config::default();
        c.set_compatibility_mode();
        assert_eq!((c.safe_flags, c.fast_nan, c.strong_mem), (2, 0, 1));
        c.set_performance_mode();
        assert_eq!((c.safe_flags, c.fast_nan, c.big_block, c.forward, c.wait), (0, 1, 2, 512, 1));
    }

    #[test]
    fn export_emits_expected_keys() {
        let c = Box64Config::default();
        let env = c.export();
        assert!(env.iter().any(|(k, v)| k == "BOX64_DYNAREC" && v == "1"));
        assert!(env.iter().any(|(k, v)| k == "BOX64_NOBANNER" && v == "1"));
        assert!(env.iter().any(|(k, v)| k == "BOX64_MMAP32" && v == "1"));
        // Empty dynaCacheFolder is not exported.
        assert!(!env.iter().any(|(k, _)| k == "BOX64_DYNACACHE_FOLDER"));
    }
}
