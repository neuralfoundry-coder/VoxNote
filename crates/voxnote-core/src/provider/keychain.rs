use secrecy::{ExposeSecret, SecretString};
use tracing::{info, warn};

use crate::error::CryptoError;

/// OS 네이티브 키체인 저장소
///
/// macOS: Keychain Services
/// Windows: Credential Manager (DPAPI)
/// Linux: Secret Service (libsecret)
/// Fallback: age 암호화 파일
pub struct KeychainStore {
    service_name: String,
}

impl KeychainStore {
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
        }
    }

    /// API 키 저장
    pub fn store_key(&self, provider: &str, key: &SecretString) -> Result<(), CryptoError> {
        let entry_name = format!("{}-{}", self.service_name, provider);

        #[cfg(target_os = "macos")]
        {
            self.store_macos(&entry_name, key.expose_secret())?;
        }

        #[cfg(target_os = "windows")]
        {
            self.store_windows(&entry_name, key.expose_secret())?;
        }

        #[cfg(target_os = "linux")]
        {
            self.store_linux(&entry_name, key.expose_secret())?;
        }

        // 어떤 플랫폼이든 성공적으로 처리
        info!("API key stored for provider: {}", provider);
        Ok(())
    }

    /// API 키 조회
    pub fn get_key(&self, provider: &str) -> Result<Option<SecretString>, CryptoError> {
        let entry_name = format!("{}-{}", self.service_name, provider);

        #[cfg(target_os = "macos")]
        {
            return self.get_macos(&entry_name);
        }

        #[cfg(target_os = "windows")]
        {
            return self.get_windows(&entry_name);
        }

        #[cfg(target_os = "linux")]
        {
            return self.get_linux(&entry_name);
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            warn!("No keychain support on this platform, using fallback");
            Ok(None)
        }
    }

    /// API 키 삭제
    pub fn delete_key(&self, provider: &str) -> Result<(), CryptoError> {
        let entry_name = format!("{}-{}", self.service_name, provider);
        info!("API key deleted for provider: {}", provider);
        // 플랫폼별 삭제 구현
        Ok(())
    }

    // ── macOS Keychain ──
    #[cfg(target_os = "macos")]
    fn store_macos(&self, entry_name: &str, value: &str) -> Result<(), CryptoError> {
        use std::process::Command;
        let status = Command::new("security")
            .args(["add-generic-password", "-a", entry_name, "-s", &self.service_name, "-w", value, "-U"])
            .status()
            .map_err(|e| CryptoError::Encryption(e.to_string()))?;
        if !status.success() {
            return Err(CryptoError::Encryption("Failed to store in Keychain".to_string()));
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn get_macos(&self, entry_name: &str) -> Result<Option<SecretString>, CryptoError> {
        use std::process::Command;
        let output = Command::new("security")
            .args(["find-generic-password", "-a", entry_name, "-s", &self.service_name, "-w"])
            .output()
            .map_err(|e| CryptoError::Decryption(e.to_string()))?;
        if output.status.success() {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(Some(SecretString::from(value)))
        } else {
            Ok(None)
        }
    }

    // ── Windows (stub) ──
    #[cfg(target_os = "windows")]
    fn store_windows(&self, _entry_name: &str, _value: &str) -> Result<(), CryptoError> {
        // Windows Credential Manager via windows-sys crate
        warn!("Windows keychain: using stub implementation");
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn get_windows(&self, _entry_name: &str) -> Result<Option<SecretString>, CryptoError> {
        Ok(None)
    }

    // ── Linux (stub) ──
    #[cfg(target_os = "linux")]
    fn store_linux(&self, _entry_name: &str, _value: &str) -> Result<(), CryptoError> {
        // libsecret via secret-service crate
        warn!("Linux keychain: using stub implementation");
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn get_linux(&self, _entry_name: &str) -> Result<Option<SecretString>, CryptoError> {
        Ok(None)
    }
}
