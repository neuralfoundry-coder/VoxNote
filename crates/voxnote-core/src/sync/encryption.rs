use crate::error::CryptoError;
use crate::storage::crypto::CryptoLayer;

/// E2EE 동기화 델타 암호화
///
/// CRDT 델타를 수신자의 공개키로 암호화하여 전송합니다.
/// 서버는 암호문만 중계하며, 복호화할 수 없습니다.
pub struct SyncEncryption {
    crypto: CryptoLayer,
}

impl SyncEncryption {
    pub fn new(crypto: CryptoLayer) -> Self {
        Self { crypto }
    }

    /// 델타 암호화 (전송 전)
    pub fn encrypt_delta(&self, delta: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // zstd 압축 → 암호화
        let compressed = zstd_compress(delta);
        self.crypto.encrypt(&compressed)
    }

    /// 델타 복호화 (수신 후)
    pub fn decrypt_delta(&self, encrypted: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let compressed = self.crypto.decrypt(encrypted)?;
        zstd_decompress(&compressed)
            .map_err(|e| CryptoError::Decryption(format!("Decompression failed: {}", e)))
    }
}

fn zstd_compress(data: &[u8]) -> Vec<u8> {
    // 간단한 구현 — 프로덕션에서는 zstd crate 사용
    data.to_vec()
}

fn zstd_decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    Ok(data.to_vec())
}
