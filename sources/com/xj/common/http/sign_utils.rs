//! Portable model of `SignUtils.b()` (both the `com.xj.common.http` and
//! `com.xj.adb.wifiui.http` copies are identical) plus a small, self-contained
//! MD5 implementation.
//!
//! The decompiled algorithm:
//!   1. put all request params into a `TreeMap` (sorted by key),
//!   2. join entries as `key=value` with `&`,
//!   3. replace every literal `null` substring with `""`,
//!   4. append `&all-egg-shell-y7ZatUDk`,
//!   5. MD5 the result and lowercase the hex digest.
//!
//! Real MD5 is implemented here in pure `std` Rust (no crypto dependency) so
//! signatures are byte-for-byte compatible with the app. See `REPORT.md`.

use std::collections::BTreeMap;

/// Appended salt used by both `SignUtils` classes.
pub const SIGN_SALT: &str = "&all-egg-shell-y7ZatUDk";

/// Compute the `sign` value for a set of request parameters.
///
/// `params` must contain `clientparams` and `time` (and any body fields) but
/// must NOT already contain `sign` — the app computes the signature before
/// inserting the `sign` key.
pub fn sign_string(params: &BTreeMap<String, String>) -> String {
    let joined = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");
    let to_hash = format!("{}{}", joined.replace("null", ""), SIGN_SALT);
    md5_hex(to_hash.as_bytes())
}

/// Variant operating on an ordered slice of pairs; keys are sorted first to
/// mirror `TreeMap` ordering.
pub fn sign_pairs(pairs: &[(String, String)]) -> String {
    let map: BTreeMap<String, String> = pairs.iter().cloned().collect();
    sign_string(&map)
}

// ---- Minimal MD5 ---------------------------------------------------------

const S: [u32; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
    5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
    4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
    6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
];

const K: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee,
    0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be,
    0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa,
    0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
    0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c,
    0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05,
    0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039,
    0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1,
    0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];

/// Return the lowercase hex MD5 digest of `input`.
pub fn md5_hex(input: &[u8]) -> String {
    let mut msg = input.to_vec();
    let bit_len = (msg.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_le_bytes());

    let (mut a0, mut b0, mut c0, mut d0) =
        (0x67452301u32, 0xefcdab89u32, 0x98badcfeu32, 0x10325476u32);

    for chunk in msg.chunks_exact(64) {
        let mut m = [0u32; 16];
        for (i, word) in m.iter_mut().enumerate() {
            *word = u32::from_le_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }

        let (mut a, mut b, mut c, mut d) = (a0, b0, c0, d0);
        for i in 0..64 {
            let (f, g) = if i < 16 {
                ((b & c) | (!b & d), i)
            } else if i < 32 {
                ((d & b) | (!d & c), (5 * i + 1) % 16)
            } else if i < 48 {
                (b ^ c ^ d, (3 * i + 5) % 16)
            } else {
                (c ^ (b | !d), (7 * i) % 16)
            };
            let tmp = d;
            d = c;
            c = b;
            b = b.wrapping_add(
                a.wrapping_add(f)
                    .wrapping_add(K[i])
                    .wrapping_add(m[g])
                    .rotate_left(S[i]),
            );
            a = tmp;
        }

        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }

    let mut out = String::with_capacity(32);
    for word in [a0, b0, c0, d0] {
        for byte in word.to_le_bytes() {
            out.push_str(&format!("{:02x}", byte));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn md5_known_vectors() {
        assert_eq!(md5_hex(b""), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(md5_hex(b"abc"), "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(md5_hex(b"message digest"), "f96b697d7cb7938d525a2f31aaf161d0");
        assert_eq!(
            md5_hex(b"abcdefghijklmnopqrstuvwxyz"),
            "c3fcd3d76192e4007dfb496cca67e13b"
        );
        assert_eq!(
            md5_hex(b"12345678901234567890123456789012345678901234567890123456789012345678901234567890"),
            "57edf4a22be3c955ac49da2e2107b67a"
        );
    }

    #[test]
    fn sign_is_sorted_and_salted() {
        let mut params = BTreeMap::new();
        params.insert("time".into(), "1000".into());
        params.insert("clientparams".into(), "cp".into());
        // "clientparams=cp&time=1000" + salt, hashed.
        let expected = md5_hex(format!("clientparams=cp&time=1000{}", SIGN_SALT).as_bytes());
        assert_eq!(sign_string(&params), expected);
        assert_eq!(sign_string(&params), sign_pairs(&[
            ("time".into(), "1000".into()),
            ("clientparams".into(), "cp".into()),
        ]));
    }

    #[test]
    fn sign_replaces_literal_null() {
        let mut a = BTreeMap::new();
        a.insert("x".into(), "null".into());
        let mut b = BTreeMap::new();
        b.insert("x".into(), "".into());
        assert_eq!(sign_string(&a), sign_string(&b));
    }
}
