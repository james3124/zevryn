//! JNI bridge for the real Android launcher UI.
//!
//! `NewStringUTF` is slot **167** of `JNINativeInterface_` (verified against
//! the official OpenJDK jni.h): 163 NewString, 164 GetStringLength,
//! 165 GetStringChars, 166 ReleaseStringChars, **167 NewStringUTF**.
//! Using 169 (GetStringUTFChars) caused the earlier SIGSEGV.
//!
//! All native data is exposed as JSON strings built by our own writer, which
//! Kotlin parses with the built-in org.json — no third-party dependency.

use std::ffi::{c_char, c_void};

const NEW_STRING_UTF_INDEX: usize = 167;
type NewStringUtf = unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void;

unsafe fn to_jstring(env: *mut c_void, text: &str) -> *mut c_void {
    if env.is_null() {
        return std::ptr::null_mut();
    }
    let interface = *(env as *const *const *const c_void);
    if interface.is_null() {
        return std::ptr::null_mut();
    }
    let new_string_utf: NewStringUtf = *(interface.add(NEW_STRING_UTF_INDEX) as *const NewStringUtf);
    let c_text = std::ffi::CString::new(text).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
    new_string_utf(env, c_text.as_ptr())
}

/// Escape a string for embedding in a JSON string literal.
pub fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

// ---- Data exposed to Kotlin -------------------------------------------------

pub const VERSION_CODE: i32 = 1;
pub const MODULE_COUNT: i32 = 45;

fn dashboard_cards_json() -> String {
    // Mirrors the two tabs in this build: Dashboard + Instant Play.
    let cards = [
        ("Recent Games", "GAMEPAD", "#FF2D6C"),
        ("Cloud Play", "CLOUD", "#1E88E5"),
        ("Free Games", "EXPLORE", "#43A047"),
        ("Find Games", "PLATFORM", "#8E24AA"),
        ("Settings", "SETTING", "#546E7A"),
    ];
    let mut out = String::from("[");
    for (i, (title, kind, color)) in cards.iter().enumerate() {
        if i > 0 { out.push(','); }
        out.push_str(&format!(
            "{{\"title\":\"{}\",\"kind\":\"{}\",\"color\":\"{}\"}}",
            json_escape(title), json_escape(kind), json_escape(color)
        ));
    }
    out.push(']');
    out
}

fn game_library_json() -> String {
    // Demo rows so the library grid renders; in a full build these would come
    // from the Room-backed GameLibraryStore.
    let games = [
        ("1", "Sample Racing", "PC"),
        ("2", "Puzzle Box", "MOBILE"),
        ("3", "Platformer", "PC"),
    ];
    let mut out = String::from("[");
    for (i, (id, title, kind)) in games.iter().enumerate() {
        if i > 0 { out.push(','); }
        out.push_str(&format!(
            "{{\"id\":\"{}\",\"title\":\"{}\",\"kind\":\"{}\"}}",
            json_escape(id), json_escape(title), json_escape(kind)
        ));
    }
    out.push(']');
    out
}

// ---- JNI exports ------------------------------------------------------------

#[no_mangle]
pub extern "C" fn Java_com_zevryn_app_MainActivity_coreVersionCode(
    _env: *mut c_void, _this: *mut c_void,
) -> i32 { VERSION_CODE }

#[no_mangle]
pub extern "C" fn Java_com_zevryn_app_MainActivity_coreModuleCount(
    _env: *mut c_void, _this: *mut c_void,
) -> i32 { MODULE_COUNT }

#[no_mangle]
pub extern "C" fn Java_com_zevryn_app_MainActivity_nativeDashboardCards(
    env: *mut c_void, _this: *mut c_void,
) -> *mut c_void { unsafe { to_jstring(env, &dashboard_cards_json()) } }

#[no_mangle]
pub extern "C" fn Java_com_zevryn_app_MainActivity_nativeGameLibrary(
    env: *mut c_void, _this: *mut c_void,
) -> *mut c_void { unsafe { to_jstring(env, &game_library_json()) } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_escape() {
        assert_eq!(super::json_escape("a\"b\\c\n"), "a\\\"b\\\\c\\n");
    }

    #[test]
    fn dashboard_json_is_valid_shape() {
        let s = dashboard_cards_json();
        assert!(s.starts_with('[') && s.ends_with(']'));
        assert!(s.contains("\"title\":\"Recent Games\""));
        assert_eq!(s.matches("\"kind\"").count(), 5);
    }

    #[test]
    fn library_json_is_valid_shape() {
        let s = game_library_json();
        assert!(s.contains("\"id\":\"1\""));
        assert_eq!(s.matches("\"title\"").count(), 3);
    }

    #[test]
    fn int_exports() {
        let p = std::ptr::null_mut();
        assert_eq!(Java_com_zevryn_app_MainActivity_coreVersionCode(p, p), VERSION_CODE);
    }

    #[test]
    fn string_exports_safe_with_null_env() {
        let p = std::ptr::null_mut();
        assert!(Java_com_zevryn_app_MainActivity_nativeDashboardCards(p, p).is_null());
    }
}
