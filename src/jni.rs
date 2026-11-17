//! Minimal JNI surface so the Rust core can be loaded by the Android shell.
//!
//! This is the bridge the APK workflow needs: a tiny `extern "C"` function
//! Kotlin can call via `System.loadLibrary("zevryn")`. The heavy lifting stays
//! in the dependency-free modules; this only reports the core version and a
//! diagnostic string derived from the local (offline) report.

/// `Java_com_zevryn_app_MainActivity_coreVersion(JNIEnv*, jobject) -> jstring`
///
/// Implemented in Kotlin-side as `external fun coreVersion(): String`.
/// We use the `#no_mangle` C ABI with the JNI naming convention; returning a
/// `const char*` is enough for a diagnostic (the Java side converts it).
#[no_mangle]
pub extern "C" fn Java_com_zevryn_app_MainActivity_coreVersion(
    _env: *mut std::ffi::c_void,
    _this: *mut std::ffi::c_void,
) -> *const std::os::raw::c_char {
    // Static C string: version of the reimplementation.
    b"zevryn-core 0.1.0\0".as_ptr() as *const std::os::raw::c_char
}

/// Diagnostic: number of wired subsystems, as a static C string.
#[no_mangle]
pub extern "C" fn Java_com_zevryn_app_MainActivity_coreStatus(
    _env: *mut std::ffi::c_void,
    _this: *mut std::ffi::c_void,
) -> *const std::os::raw::c_char {
    b"core loaded (offline-capable)\0".as_ptr() as *const std::os::raw::c_char
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn version_string_is_valid_cstr() {
        let p = Java_com_zevryn_app_MainActivity_coreVersion(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        let s = unsafe { CStr::from_ptr(p) }.to_str().unwrap();
        assert_eq!(s, "zevryn-core 0.1.0");
    }

    #[test]
    fn status_string_is_valid_cstr() {
        let p = Java_com_zevryn_app_MainActivity_coreStatus(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        let s = unsafe { CStr::from_ptr(p) }.to_str().unwrap();
        assert!(s.contains("offline-capable"));
    }
}
