use argon2::Argon2;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use secrecy::{ExposeSecret, SecretString};
use zeroize::Zeroize;

use crate::error::CryptoError;

/// E2EE 암호화 레이어
///
/// Argon2id로 패스워드에서 키를 파생하고,
/// ChaCha20-Poly1305로 데이터를 암호화/복호화합니다.
pub struct CryptoLayer {
    cipher: ChaCha20Poly1305,
}

/// Argon2id 파라미터
const ARGON2_TIME_COST: u32 = 3;
const ARGON2_MEMORY_COST: u32 = 65536; // 64MB
const ARGON2_PARALLELISM: u32 = 4;
const KEY_LENGTH: usize = 32;

impl CryptoLayer {
    /// 패스워드와 솔트에서 암호화 키를 파생하여 생성
    pub fn from_password(password: &SecretString, salt: &[u8]) -> Result<Self, CryptoError> {
        let mut key = [0u8; KEY_LENGTH];

        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(
                ARGON2_MEMORY_COST,
                ARGON2_TIME_COST,
                ARGON2_PARALLELISM,
                Some(KEY_LENGTH),
            )
            .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?,
        );

        argon2
            .hash_password_into(
                password.expose_secret().as_bytes(),
                salt,
                &mut key,
            )
            .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;

        let cipher = ChaCha20Poly1305::new_from_slice(&key)
            .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;

        // 키 메모리 즉시 제로화
        key.zeroize();

        Ok(Self { cipher })
    }

    /// 원시 키에서 직접 생성 (32바이트)
    pub fn from_key(key: &[u8; 32]) -> Result<Self, CryptoError> {
        let cipher = ChaCha20Poly1305::new_from_slice(key)
            .map_err(|_| CryptoError::InvalidKey)?;
        Ok(Self { cipher })
    }

    /// 데이터 암호화
    ///
    /// 반환: nonce(12bytes) + ciphertext
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let nonce_bytes: [u8; 12] = rand_nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| CryptoError::Encryption(e.to_string()))?;

        // nonce를 앞에 붙여서 반환
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend(ciphertext);
        Ok(result)
    }

    /// 데이터 복호화
    ///
    /// 입력: nonce(12bytes) + ciphertext
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if data.len() < 12 {
            return Err(CryptoError::Decryption("Data too short".to_string()));
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CryptoError::Decryption(e.to_string()))
    }
}

/// 랜덤 nonce 생성 (12바이트)
///
/// 타임스탬프 + 원자 카운터 + PID로 고유성 보장
fn rand_nonce() -> [u8; 12] {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let mut nonce = [0u8; 12];
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let ts_bytes = ts.to_le_bytes();
    nonce[..6].copy_from_slice(&ts_bytes[..6]);

    // 원자 카운터 (프로세스 내 고유성 보장)
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    nonce[6..10].copy_from_slice(&(count as u32).to_le_bytes());

    // PID (프로세스 간 고유성)
    let pid = std::process::id() as u16;
    nonce[10..12].copy_from_slice(&pid.to_le_bytes());

    nonce
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::SecretString;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let password = SecretString::from("test-password-123".to_string());
        let salt = b"voxnote-test-salt-16b";

        let crypto = CryptoLayer::from_password(&password, salt).unwrap();

        let plaintext = b"Hello, VoxNote! This is a secret message.";
        let encrypted = crypto.encrypt(plaintext).unwrap();

        // 암호문은 원문과 달라야 함
        assert_ne!(&encrypted[12..], plaintext.as_slice());

        let decrypted = crypto.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_password_fails() {
        let salt = b"voxnote-test-salt-16b";

        let crypto1 = CryptoLayer::from_password(
            &SecretString::from("password1".to_string()),
            salt,
        )
        .unwrap();

        let crypto2 = CryptoLayer::from_password(
            &SecretString::from("password2".to_string()),
            salt,
        )
        .unwrap();

        let encrypted = crypto1.encrypt(b"secret data").unwrap();
        assert!(crypto2.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_from_key() {
        let key = [42u8; 32];
        let crypto = CryptoLayer::from_key(&key).unwrap();

        let encrypted = crypto.encrypt(b"test").unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, b"test");
    }
}
