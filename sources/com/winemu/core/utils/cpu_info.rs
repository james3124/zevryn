//! Rust model of the portable parts of
//! `com.winemu.core.utils.CpuInfoCollector` and the `WINEMU_CPU_AFFINITY`
//! computation in `com.winemu.core.controller.EnvironmentController`
//! (GameHub Lite v5.1.8).
//!
//! Two behaviors are portable and kept here:
//!
//! 1. **Core counting.** `CpuInfoCollector.b()` counts files in
//!    `/sys/devices/system/cpu/` whose names match `cpu[0-9]`
//!    (`CpuInfoCollector.c()`), caches the result in static `f20888b`, and
//!    falls back to `availableProcessors()`. The sysfs scan and caching are
//!    filesystem/OS concerns; the pure name predicate and "count from names"
//!    step are modeled in [`is_cpu_dir_name`] / [`core_count_from_names`].
//!
//! 2. **Affinity mask.** `EnvironmentController.d()` computes
//!    `~((1 << iB) - 1)` where `iB = coreCount - reservedCores`, but only when
//!    `reservedCores > 0` and `iB > 0`, and writes it to `WINEMU_CPU_AFFINITY`.
//!    Modeled in [`cpu_affinity`] / [`set_cpu_affinity`].
//!
//! The `AllCoreFrequencyInfo` percentage math (`d()` / `e()`) is also pure and
//! modeled by [`AllCoreFrequencyInfo`]. Reading `/sys/.../cpufreq/*` files
//! (`f()`, `i()`, `j()`, `k()`, `g()`, `h()`) is real filesystem I/O and out of
//! scope.

use super::EnvVars;

/// Environment key written by `EnvironmentController.d()`.
pub const WINEMU_CPU_AFFINITY: &str = "WINEMU_CPU_AFFINITY";

/// `CpuInfoCollector.c()` — true only for names matching the regex
/// `cpu[0-9]`: exactly `"cpu"` followed by a single ASCII digit. Note that
/// `"cpu10"` does **not** match (the Java regex is anchored by `Pattern.matches`).
pub fn is_cpu_dir_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    bytes.len() == 4
        && &bytes[..3] == b"cpu"
        && bytes[3].is_ascii_digit()
}

/// Count core directory names that match `cpu[0-9]` — the pure result of
/// `CpuInfoCollector.b()`'s `listFiles(...).length` step.
pub fn core_count_from_names<'a, I>(names: I) -> usize
where
    I: IntoIterator<Item = &'a str>,
{
    names.into_iter().filter(|n| is_cpu_dir_name(n)).count()
}

/// sysfs paths used by `CpuInfoCollector.i()/j()/k()` for a core index.
pub mod sysfs {
    /// `cpufreq/scaling_cur_freq` path (`i()`).
    pub fn cur_freq(core: usize) -> String {
        format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_cur_freq", core)
    }
    /// `cpufreq/cpuinfo_max_freq` path (`j()`).
    pub fn max_freq(core: usize) -> String {
        format!("/sys/devices/system/cpu/cpu{}/cpufreq/cpuinfo_max_freq", core)
    }
    /// `cpufreq/cpuinfo_min_freq` path (`k()`).
    pub fn min_freq(core: usize) -> String {
        format!("/sys/devices/system/cpu/cpu{}/cpufreq/cpuinfo_min_freq", core)
    }
}

/// Result of `EnvironmentController.d()`'s affinity computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Affinity {
    /// Total available cores (`CpuInfoCollector.INSTANCE.b()`).
    pub available_cores: i32,
    /// Cores reserved for the host (`container.x()`).
    pub reserved_cores: i32,
    /// `iB = available_cores - reserved_cores`, the number of low cores masked
    /// out. Guaranteed `> 0` for any produced `Affinity`.
    pub shift: i32,
    /// Java `~((1 << iB) - 1)` interpreted as a signed 32-bit `int`.
    /// e.g. 8 cores, 2 reserved → `-64`.
    pub mask: i32,
    /// The same mask viewed as unsigned 32 bits (e.g. `0xFFFF_FFC0`).
    pub mask_u32: u32,
}

/// `EnvironmentController.d()`: compute the CPU affinity mask, or `None` when
/// the controller would return early (`reserved_cores <= 0` or `iB <= 0`).
pub fn cpu_affinity(available_cores: i32, reserved_cores: i32) -> Option<Affinity> {
    if reserved_cores <= 0 {
        return None;
    }
    let ib = available_cores - reserved_cores;
    if ib <= 0 {
        return None;
    }
    // Java `int` is 32-bit and shift distances wrap modulo 32. For normal
    // inputs (0 < iB < 32) this is simply (1u32 << iB) - 1.
    let shift_bits = (ib as u32) & 31;
    let low_mask = (1u32 << shift_bits).wrapping_sub(1);
    let mask_u32 = !low_mask;
    Some(Affinity {
        available_cores,
        reserved_cores,
        shift: ib,
        mask: mask_u32 as i32,
        mask_u32,
    })
}

/// Apply the computed affinity to an [`EnvVars`] map under
/// `WINEMU_CPU_AFFINITY`, mirroring `envVars.c("WINEMU_CPU_AFFINITY",
/// Integer.valueOf(...))`. Returns the affinity that was written, or `None`
/// if the controller would have skipped the write.
pub fn set_cpu_affinity(
    env: &mut EnvVars,
    available_cores: i32,
    reserved_cores: i32,
) -> Option<Affinity> {
    let affinity = cpu_affinity(available_cores, reserved_cores)?;
    env.set(WINEMU_CPU_AFFINITY, affinity.mask);
    Some(affinity)
}

/// `CpuInfoCollector.AllCoreFrequencyInfo` — three parallel per-core arrays
/// (`a()` = current, `c()` = min, `b()` = max). Kept as plain vectors.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AllCoreFrequencyInfo {
    /// `f20889a` — `scaling_cur_freq` per core (`a()`).
    pub cur: Vec<i32>,
    /// `f20890b` — `cpuinfo_min_freq` per core (`c()`).
    pub min: Vec<i32>,
    /// `f20891c` — `cpuinfo_max_freq` per core (`b()`).
    pub max: Vec<i32>,
}

impl AllCoreFrequencyInfo {
    /// `new(int size)` — three zero-filled arrays of `size`.
    pub fn new(size: usize) -> Self {
        Self {
            cur: vec![0; size],
            min: vec![0; size],
            max: vec![0; size],
        }
    }

    /// `e(cur, min, max)` — `(cur - min) * 100 / (max - min)`, or `0` when
    /// `max <= 0` or `max <= min`. Uses i64 arithmetic like the Java cast.
    pub fn usage_percent(cur: i32, min: i32, max: i32) -> i32 {
        let span = max - min;
        if max > 0 && span > 0 {
            (((cur - min) as i64 * 100) / span as i64) as i32
        } else {
            0
        }
    }

    /// `d(fi)` — a length+1 array: index 0 is the aggregate percentage of the
    /// summed frequencies, indices 1.. are each core's percentage.
    pub fn usage_with_aggregate(&self) -> Vec<i32> {
        let len = self.cur.len();
        let mut out = Vec::with_capacity(len + 1);
        for i in 0..len {
            out.push(Self::usage_percent(self.cur[i], self.min[i], self.max[i]));
        }
        let sum_cur: i32 = self.cur.iter().sum();
        let sum_min: i32 = self.min.iter().sum();
        let sum_max: i32 = self.max.iter().sum();
        out.insert(0, Self::usage_percent(sum_cur, sum_min, sum_max));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_dir_name_regex() {
        assert!(is_cpu_dir_name("cpu0"));
        assert!(is_cpu_dir_name("cpu9"));
        assert!(!is_cpu_dir_name("cpu10")); // single digit only
        assert!(!is_cpu_dir_name("cpu"));
        assert!(!is_cpu_dir_name("cpuA"));
        assert!(!is_cpu_dir_name("cpu0x"));
    }

    #[test]
    fn counts_cpu_dir_names() {
        let names = ["cpu0", "cpu1", "cpu2", "cpu10", "cpufreq", "kernel"];
        assert_eq!(core_count_from_names(names.iter().copied()), 3);
    }

    #[test]
    fn sysfs_paths() {
        assert_eq!(
            sysfs::cur_freq(3),
            "/sys/devices/system/cpu/cpu3/cpufreq/scaling_cur_freq"
        );
        assert_eq!(
            sysfs::max_freq(0),
            "/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq"
        );
        assert_eq!(
            sysfs::min_freq(7),
            "/sys/devices/system/cpu/cpu7/cpufreq/cpuinfo_min_freq"
        );
    }

    #[test]
    fn affinity_mask_matches_java() {
        // 8 - 2 = 6 → ~((1<<6)-1) = ~63 = -64 = 0xFFFFFFC0
        let a = cpu_affinity(8, 2).unwrap();
        assert_eq!(a.shift, 6);
        assert_eq!(a.mask, -64);
        assert_eq!(a.mask_u32, 0xFFFF_FFC0);

        // 4 - 1 = 3 → ~7 = -8
        let b = cpu_affinity(4, 1).unwrap();
        assert_eq!(b.mask, -8);
        assert_eq!(b.mask_u32, 0xFFFF_FFF8);

        // 16 - 4 = 12 → ~0xFFF = -4096
        let c = cpu_affinity(16, 4).unwrap();
        assert_eq!(c.mask, -4096);
    }

    #[test]
    fn affinity_returns_none_for_invalid_inputs() {
        assert_eq!(cpu_affinity(8, 0), None);
        assert_eq!(cpu_affinity(8, -1), None);
        assert_eq!(cpu_affinity(2, 4), None); // iB <= 0
        assert_eq!(cpu_affinity(4, 4), None); // iB == 0
    }

    #[test]
    fn set_cpu_affinity_writes_env() {
        let mut env = EnvVars::new();
        let a = set_cpu_affinity(&mut env, 8, 2).unwrap();
        assert_eq!(env.get(WINEMU_CPU_AFFINITY), Some("-64"));
        assert_eq!(a.mask_u32, 0xFFFF_FFC0);
        assert_eq!(env.export(), vec!["WINEMU_CPU_AFFINITY=-64"]);

        // controller would skip the write
        let mut env2 = EnvVars::new();
        assert!(set_cpu_affinity(&mut env2, 8, 0).is_none());
        assert!(env2.is_empty());
    }

    #[test]
    fn frequency_usage_math() {
        // e(cur=75, min=50, max=100) = 25*100/50 = 50
        assert_eq!(AllCoreFrequencyInfo::usage_percent(75, 50, 100), 50);
        assert_eq!(AllCoreFrequencyInfo::usage_percent(50, 50, 100), 0);
        assert_eq!(AllCoreFrequencyInfo::usage_percent(0, 0, 0), 0);
        assert_eq!(AllCoreFrequencyInfo::usage_percent(10, 20, 100), -12); // negative ok

        let fi = AllCoreFrequencyInfo {
            cur: vec![75, 100],
            min: vec![50, 50],
            max: vec![100, 100],
        };
        // aggregate: (175-100)*100/(200-100) = 75
        assert_eq!(fi.usage_with_aggregate(), vec![75, 50, 100]);
    }
}
