//! Rust model of the `com.xj.common.preference` package (portable logic only).
//!
//! Mirrors `sources/com/xj/common/data/preferences/` (AppPreferences,
//! MMKVExtensionsKt), `com/xj/base/language/GHLocaleManager` and the
//! SharedPreferences usage in `com/xj/app/SplashActivity` from GameHub Lite
//! v5.1.8. Only the portable key/value semantics are modeled here: exact
//! string keys, value types and Java initializer defaults. Android
//! `SharedPreferences`, Tencent MMKV (native `libmmkv.so`) and the Kotlin
//! delegated-property machinery are intentionally left out — see `REPORT.md`.

pub mod keys;
pub mod kv;
pub mod migration;
pub mod settings;

pub use self::keys::SettingsKey;
pub use self::kv::{InMemoryKv, KvStore, Value};
pub use self::migration::{migrate, migrate_all, LEGACY_KEY_MAP};
pub use self::settings::Settings;
