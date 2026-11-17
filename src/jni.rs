//! JNI surface for the Android shell.
//!
//! The Kotlin side declares `external fun coreVersion(): String`, so these
//! functions MUST return a real `jstring` (jobject), not a C `char*`. We call
//! `NewStringUTF` through the JNI function table (index 169) to build it.
//! Returning a raw `const char*` here is what made the app crash on launch.

use std::ffi::{c_char, c_void};

/// JNIEnv is a pointer to a pointer to the JNINativeInterface_ function table.
/// NewStringUTF is slot 169 in that table (JNI 1.1 ordering).
const NEW_STRING_UTF_INDEX: usize = 169;

type NewStringUtf = unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void;

/// Pure Rust side of the version string (testable without a JVM).
pub fn version_str() -> &'static str {
    "zevryn-core 0.1.0"
}

/// Pure Rust side of the status string (testable without a JVM).
pub fn status_str() -> &'static str {
    "core loaded (offline-capable)"
}

unsafe fn to_jstring(env: *mut c_void, text: &str) -> *mut c_void {
    if env.is_null() {
        return std::ptr::null_mut();
    }
    // JNIEnv* -> JNINativeInterface_**
    let interface = *(env as *const *const *const c_void);
    if interface.is_null() {
        return std::ptr::null_mut();
    }
    let slot = interface.add(NEW_STRING_UTF_INDEX) as *const NewStringUtf;
    let new_string_utf = *slot;
    if new_string_utf.is_null() {
        return std::ptr::null_mut();
    }
    let c_text = std::ffi::CString::new(text).unwrap_or_default();
    new_string_utf(env, c_text.as_ptr())
}

#[no_mangle]
pub extern "C" fn Java_com_zevryn_app_MainActivity_coreVersion(
    env: *mut c_void,
    _this: *mut c_void,
) -> *mut c_void {
    unsafe { to_jstring(env, version_str()) }
}

#[no_mangle]
pub extern "C" fn Java_com_zevryn_app_MainActivity_coreStatus(
    env: *mut c_void,
    _this: *mut c_void,
) -> *mut c_void {
    unsafe { to_jstring(env, status_str()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_string() {
        assert_eq!(version_str(), "zevryn-core 0.1.0");
    }

    #[test]
    fn status_string() {
        assert!(status_str().contains("offline-capable"));
    }

    #[test]
    fn null_env_is_safe() {
        let p = Java_com_zevryn_app_MainActivity_coreVersion(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        assert!(p.is_null());
    }
}
