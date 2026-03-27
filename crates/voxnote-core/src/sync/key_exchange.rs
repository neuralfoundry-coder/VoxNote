use serde::{Deserialize, Serialize};

/// 디바이스 키 쌍 및 교환 관리
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceKeyPair {
    pub device_id: String,
    pub public_key: Vec<u8>,
    // private_key는 OS 키체인에 저장
}

/// 디바이스 페어링 코드 생성
pub fn generate_pairing_code(device_id: &str, public_key: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(device_id.as_bytes());
    hasher.update(public_key);
    let hash = hasher.finalize();
    // 6자리 숫자 코드
    let num = u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]);
    format!("{:06}", num % 1000000)
}

/// 디바이스 키 쌍 생성
pub fn generate_device_keypair(device_id: &str) -> DeviceKeyPair {
    // 프로덕션에서는 X25519 키 쌍 생성
    // 현재는 플레이스홀더
    DeviceKeyPair {
        device_id: device_id.to_string(),
        public_key: vec![0u8; 32],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pairing_code() {
        let code = generate_pairing_code("device-1", &[1, 2, 3]);
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }
}
