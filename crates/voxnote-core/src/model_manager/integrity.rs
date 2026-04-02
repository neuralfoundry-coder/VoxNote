use sha2::{Digest, Sha256};
use std::path::Path;
use tracing::debug;

use crate::error::ModelError;

/// SHA-256 해시 검증
///
/// 파일의 SHA-256 해시가 기대값과 일치하는지 확인합니다.
pub fn verify_sha256(path: &Path, expected: &str) -> Result<bool, ModelError> {
    // placeholder 해시는 검증 건너뛰기 (아직 실제 해시가 설정되지 않은 모델)
    if expected.is_empty() || expected.starts_with("placeholder") {
        debug!("SHA-256 verification skipped (placeholder): {:?}", path);
        return Ok(true);
    }

    let hash = compute_sha256(path)?;

    if hash == expected.to_lowercase() {
        debug!("SHA-256 verified: {:?}", path);
        Ok(true)
    } else {
        Err(ModelError::IntegrityCheck {
            expected: expected.to_string(),
            actual: hash,
        })
    }
}

/// 파일의 SHA-256 해시 계산
pub fn compute_sha256(path: &Path) -> Result<String, ModelError> {
    use std::io::Read;

    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let hash = hasher.finalize();
    Ok(hex::encode(hash))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_compute_sha256() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bin");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"hello voxnote").unwrap();

        let hash = compute_sha256(&path).unwrap();
        assert_eq!(hash.len(), 64); // SHA-256 = 32 bytes = 64 hex chars
    }

    #[test]
    fn test_verify_sha256() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bin");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"hello voxnote").unwrap();

        let hash = compute_sha256(&path).unwrap();
        assert!(verify_sha256(&path, &hash).unwrap());
        assert!(verify_sha256(&path, "wrong_hash").is_err());
    }
}
