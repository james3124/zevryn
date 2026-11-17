//! Aria download-task configuration with defaults taken from
//! `aria_config.xml` (the `<download>` section).
//!
//! The reference asset (`apktool/assets/aria_config.xml`) overrides Aria's
//! framework defaults with, notably: `threadNum=3`, `reTryNum=3`,
//! `reTryInterval=5000`, `useBlock=true`, `maxSpeed=0` (unlimited),
//! `useHeadRequest=false`, `maxTaskNum=2`, `connectTimeOut=5000`,
//! `iOTimeOut=10000`, `buffSize=8192`, `queueMod=wait`, `updateInterval=1000`.
//! The accompanying XML comments state the invariants modelled by
//! [`AriaTaskConfig::sanitized`]: threads >= 1, buffer >= 2048, IO timeout >=
//! 10000 ms, and that the thread setting is ignored for archives under 1 MiB.
//!
//! `net.lingala.zip4j` / `UnzipService` use an 8192-byte buffer as well
//! (`WinEmuFileExplorer.F(unzipFilePath, destPath, 8192, ...)`, and the
//! `UnzipServiceConfig` default of 3 retries), which matches `buffSize` here.

use std::fmt;

/// Aria queue execution mode (`com.arialyy.aria.core.QueueMod`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueueMod {
    /// Tasks wait for their turn in the queue (the configured default).
    Wait,
    /// Tasks run immediately.
    Run,
}

impl QueueMod {
    pub const fn as_str(self) -> &'static str {
        match self {
            QueueMod::Wait => "wait",
            QueueMod::Run => "run",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "wait" => Some(QueueMod::Wait),
            "run" => Some(QueueMod::Run),
            _ => None,
        }
    }
}

impl fmt::Display for QueueMod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Settings applied when starting an Aria download task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AriaTaskConfig {
    /// Number of download threads (`threadNum`); minimum 1.
    pub threads: u32,
    /// Retry count on failure (`reTryNum`).
    pub retry: u32,
    /// Delay between retries in milliseconds (`reTryInterval`).
    pub retry_interval_ms: u64,
    /// Whether multi-threaded block download mode is enabled (`useBlock`).
    pub use_block: bool,
    /// Speed cap in KB/s (`maxSpeed`); 0 means unlimited.
    pub max_speed_kb: u64,
    /// Whether to use HEAD requests to discover file size (`useHeadRequest`).
    pub use_head_request: bool,
    /// Maximum concurrent tasks in the queue (`maxTaskNum`).
    pub max_task_num: u32,
    /// URL connection timeout in milliseconds (`connectTimeOut`).
    pub connect_timeout_ms: u64,
    /// IO read timeout in milliseconds (`iOTimeOut`); minimum 10000.
    pub io_timeout_ms: u64,
    /// File write buffer size in bytes (`buffSize`); minimum 2048.
    pub buff_size: u32,
    /// Queue execution mode (`queueMod`).
    pub queue_mod: QueueMod,
    /// Progress update interval in milliseconds (`updateInterval`).
    pub update_interval_ms: u64,
}

impl AriaTaskConfig {
    /// Smallest task for which the multi-thread setting takes effect (1 MiB).
    pub const SMALL_TASK_BYTES: u64 = 1024 * 1024;
    pub const MIN_THREADS: u32 = 1;
    pub const MIN_BUFF_SIZE: u32 = 2048;
    pub const MIN_IO_TIMEOUT_MS: u64 = 10_000;

    /// Defaults matching the `<download>` block of `aria_config.xml`.
    pub const DEFAULT: Self = Self {
        threads: 3,
        retry: 3,
        retry_interval_ms: 5000,
        use_block: true,
        max_speed_kb: 0,
        use_head_request: false,
        max_task_num: 2,
        connect_timeout_ms: 5000,
        io_timeout_ms: 10_000,
        buff_size: 8192,
        queue_mod: QueueMod::Wait,
        update_interval_ms: 1000,
    };

    /// Apply the invariants documented in `aria_config.xml` so a config built
    /// from arbitrary/partial XML cannot violate Aria's constraints.
    pub fn sanitized(mut self) -> Self {
        self.threads = self.threads.max(Self::MIN_THREADS);
        self.buff_size = self.buff_size.max(Self::MIN_BUFF_SIZE);
        self.io_timeout_ms = self.io_timeout_ms.max(Self::MIN_IO_TIMEOUT_MS);
        self
    }

    /// Effective number of threads for a task of `total_bytes`.
    ///
    /// The config comment states that `threadNum` is ignored for tasks smaller
    /// than 1 MiB; such tasks use a single thread.
    pub fn effective_threads(&self, total_bytes: u64) -> u32 {
        if total_bytes < Self::SMALL_TASK_BYTES {
            1
        } else {
            self.threads.max(Self::MIN_THREADS)
        }
    }

    /// Parse the `<key value="..." />` tags of an aria XML file, overriding
    /// only the values that are present. Unknown or malformed values fall back
    /// to [`AriaTaskConfig::DEFAULT`]. The result is sanitized.
    pub fn from_xml(xml: &str) -> Self {
        let mut cfg = Self::DEFAULT;
        macro_rules! set_int {
            ($tag:literal, $field:ident) => {
                if let Some(v) = tag_value(xml, $tag).and_then(|v| v.parse().ok()) {
                    cfg.$field = v;
                }
            };
        }
        set_int!("threadNum", threads);
        set_int!("reTryNum", retry);
        set_int!("reTryInterval", retry_interval_ms);
        set_int!("maxSpeed", max_speed_kb);
        set_int!("maxTaskNum", max_task_num);
        set_int!("connectTimeOut", connect_timeout_ms);
        set_int!("iOTimeOut", io_timeout_ms);
        set_int!("buffSize", buff_size);
        set_int!("updateInterval", update_interval_ms);

        if let Some(v) = tag_value(xml, "useBlock").and_then(|v| parse_bool(&v)) {
            cfg.use_block = v;
        }
        if let Some(v) = tag_value(xml, "useHeadRequest").and_then(|v| parse_bool(&v)) {
            cfg.use_head_request = v;
        }
        if let Some(v) = tag_value(xml, "queueMod").and_then(|v| QueueMod::from_str(&v)) {
            cfg.queue_mod = v;
        }

        cfg.sanitized()
    }
}

impl Default for AriaTaskConfig {
    fn default() -> Self {
        Self::DEFAULT
    }
}

fn parse_bool(v: &str) -> Option<bool> {
    match v.trim() {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

/// Minimal reader for `<tag value="..." />` (also accepts `<tag>...</tag>`).
fn tag_value(xml: &str, key: &str) -> Option<String> {
    let open = format!("<{key}");
    let start = xml.find(&open)?;
    let tag_end = xml[start..].find('>')? + start;
    let tag = &xml[start..tag_end];
    if let Some(v) = attr_value(tag, "value") {
        return Some(v);
    }
    let close = format!("</{key}>");
    let body_start = tag_end + 1;
    let end = xml[body_start..].find(&close)? + body_start;
    Some(xml[body_start..end].trim().to_owned())
}

fn attr_value(tag: &str, name: &str) -> Option<String> {
    let pat = format!("{name}=\"");
    let start = tag.find(&pat)? + pat.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const REFERENCE_ASSET: &str = "/home/Project/Zevryn/reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/apktool/assets/aria_config.xml";

    #[test]
    fn defaults_match_aria_config_xml() {
        let c = AriaTaskConfig::default();
        assert_eq!(c.threads, 3);
        assert_eq!(c.retry, 3);
        assert_eq!(c.retry_interval_ms, 5000);
        assert!(c.use_block);
        assert_eq!(c.max_speed_kb, 0);
        assert!(!c.use_head_request);
        assert_eq!(c.max_task_num, 2);
        assert_eq!(c.connect_timeout_ms, 5000);
        assert_eq!(c.io_timeout_ms, 10_000);
        assert_eq!(c.buff_size, 8192);
        assert_eq!(c.queue_mod, QueueMod::Wait);
        assert_eq!(c.update_interval_ms, 1000);
    }

    #[test]
    fn parses_attribute_form_and_overrides_defaults() {
        let xml = r#"<aria><download><threadNum value="8" /><reTryNum value="1" />
            <useBlock value="false" /><queueMod value="run" />
            <iOTimeOut value="30000" /></download></aria>"#;
        let c = AriaTaskConfig::from_xml(xml);
        assert_eq!(c.threads, 8);
        assert_eq!(c.retry, 1);
        assert!(!c.use_block);
        assert_eq!(c.queue_mod, QueueMod::Run);
        assert_eq!(c.io_timeout_ms, 30_000);
        // Unspecified fields keep defaults.
        assert_eq!(c.buff_size, 8192);
    }

    #[test]
    fn sanitizes_invalid_values() {
        let xml = r#"<download><threadNum value="0" /><buffSize value="16" />
            <iOTimeOut value="100" /></download>"#;
        let c = AriaTaskConfig::from_xml(xml);
        assert_eq!(c.threads, 1);
        assert_eq!(c.buff_size, 2048);
        assert_eq!(c.io_timeout_ms, 10_000);
    }

    #[test]
    fn threads_ignored_for_small_tasks() {
        let c = AriaTaskConfig::default();
        assert_eq!(c.effective_threads(AriaTaskConfig::SMALL_TASK_BYTES - 1), 1);
        assert_eq!(c.effective_threads(AriaTaskConfig::SMALL_TASK_BYTES), 3);
    }

    #[test]
    fn parses_real_asset_when_present() {
        let Ok(xml) = std::fs::read_to_string(REFERENCE_ASSET) else {
            return;
        };
        assert_eq!(AriaTaskConfig::from_xml(&xml), AriaTaskConfig::DEFAULT);
    }
}
