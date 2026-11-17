//! Portable model of the cryptographic algorithms shipped by GameHub Lite
//! v5.1.8 through `org.bouncycastle`, `org.conscrypt` and the JDK `javax.crypto`
//! / `java.security` providers.
//!
//! These are third-party / platform libraries. They are **not reimplemented**
//! here; this module only records the algorithm inventory and the Rust crates
//! that would be used to replace them. See `apktool/lib/REPORT.md`.

use std::fmt;

/// Cryptographic algorithms/primitives present in the reference source tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CryptoAlgorithm {
    /// Advanced Encryption Standard (AES, incl. GCM/CBC via Bouncy Castle).
    Aes,
    /// ChaCha20 stream cipher (RFC 7539 variant used by conscrypt/TLS).
    ChaCha20,
    /// RSA public-key encryption / signatures.
    Rsa,
    /// SHA-1 message digest.
    Sha1,
    /// HMAC (keyed message authentication code).
    Hmac,
    /// HPKE (Hybrid Public Key Encryption) from conscrypt.
    Hpke,
    /// Post-quantum cryptography (CRYSTALS-Kyber/Dilithium, etc.).
    Pqc,
}

impl CryptoAlgorithm {
    /// Every algorithm, in a stable order.
    pub const ALL: [CryptoAlgorithm; 7] = [
        CryptoAlgorithm::Aes,
        CryptoAlgorithm::ChaCha20,
        CryptoAlgorithm::Rsa,
        CryptoAlgorithm::Sha1,
        CryptoAlgorithm::Hmac,
        CryptoAlgorithm::Hpke,
        CryptoAlgorithm::Pqc,
    ];

    /// Canonical short name of the algorithm.
    pub const fn name(self) -> &'static str {
        match self {
            CryptoAlgorithm::Aes => "AES",
            CryptoAlgorithm::ChaCha20 => "ChaCha20",
            CryptoAlgorithm::Rsa => "RSA",
            CryptoAlgorithm::Sha1 => "SHA1",
            CryptoAlgorithm::Hmac => "HMAC",
            CryptoAlgorithm::Hpke => "HPKE",
            CryptoAlgorithm::Pqc => "PQC",
        }
    }

    /// Suggested Rust crate that replaces the corresponding Java implementation.
    pub const fn suggested_crate(self) -> &'static str {
        match self {
            CryptoAlgorithm::Aes => "aes-gcm",
            CryptoAlgorithm::ChaCha20 => "chacha20poly1305",
            CryptoAlgorithm::Rsa => "rsa",
            CryptoAlgorithm::Sha1 => "sha1",
            CryptoAlgorithm::Hmac => "hmac",
            CryptoAlgorithm::Hpke => "hpke",
            CryptoAlgorithm::Pqc => "ml-kem",
        }
    }

    /// Original Java class/package that provided this algorithm in the reference.
    pub const fn origin(self) -> &'static str {
        match self {
            CryptoAlgorithm::Aes => "org.bouncycastle.crypto.engines.AESEngine",
            CryptoAlgorithm::ChaCha20 => {
                "org.bouncycastle.crypto.engines.ChaCha7539Engine / org.conscrypt"
            }
            CryptoAlgorithm::Rsa => "org.bouncycastle.crypto.engines.RSABlindedEngine",
            CryptoAlgorithm::Sha1 => "org.bouncycastle.crypto.digests.SHA1Digest",
            CryptoAlgorithm::Hmac => "org.bouncycastle.crypto.macs.HMac / javax.crypto.Mac",
            CryptoAlgorithm::Hpke => "org.conscrypt.HpkeImpl / HpkeSpi",
            CryptoAlgorithm::Pqc => "org.bouncycastle.pqc.crypto.crystals",
        }
    }
}

impl fmt::Display for CryptoAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn models_all_seven_algorithms() {
        assert_eq!(CryptoAlgorithm::ALL.len(), 7);
        let mut unique = std::collections::BTreeSet::new();
        for alg in CryptoAlgorithm::ALL {
            assert!(unique.insert(alg));
            assert!(!alg.name().is_empty());
            assert!(!alg.suggested_crate().is_empty());
            assert!(!alg.origin().is_empty());
        }
    }

    #[test]
    fn crate_mapping_matches_expected_crates() {
        let expected = [
            "aes-gcm",
            "chacha20poly1305",
            "rsa",
            "sha1",
            "hmac",
            "hpke",
            "ml-kem",
        ];
        for (alg, want) in CryptoAlgorithm::ALL.iter().zip(expected.iter()) {
            assert_eq!(alg.suggested_crate(), *want);
        }
    }

    #[test]
    fn display_matches_name() {
        for alg in CryptoAlgorithm::ALL {
            assert_eq!(alg.to_string(), alg.name());
        }
        assert_eq!(CryptoAlgorithm::ChaCha20.to_string(), "ChaCha20");
    }
}
