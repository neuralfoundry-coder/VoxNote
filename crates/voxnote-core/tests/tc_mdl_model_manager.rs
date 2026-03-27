//! TC-MDL: 모델 매니저 통합 테스트
//! 관련 요구사항: FR-MDL-001 ~ FR-MDL-004
//! 참조: docs/test/test-cases/TC-MDL.md

use voxnote_core::model_manager::registry::{ModelRegistry, ModelType};
use voxnote_core::model_manager::integrity;
use voxnote_core::model_manager::disk::DiskManager;
use std::io::Write;
use tempfile::TempDir;

// ── TC-MDL-001-01: registry.toml 카탈로그 파싱 ─────────────────

#[test]
fn tc_mdl_001_01_parse_valid_registry() {
    let toml = r#"
[[model]]
id = "whisper-tiny-q8"
name = "Whisper Tiny"
type = "stt"
size_bytes = 78000000
quantization = "Q8_0"
languages = ["auto", "ko", "en"]
min_ram_mb = 512
gpu_recommended = false
download_url = "https://example.com/whisper-tiny.bin"
sha256 = "abc123def456"

[[model]]
id = "qwen-7b-q4"
name = "Qwen 7B"
type = "llm"
size_bytes = 4368000000
languages = ["ko", "en"]
min_ram_mb = 8192
gpu_recommended = true
download_url = "https://example.com/qwen-7b.gguf"
sha256 = "789xyz"
description = "Meeting summary model"
"#;
    let registry = ModelRegistry::parse(toml).unwrap();
    assert_eq!(registry.models.len(), 2);

    let stt = registry.models_by_type(&ModelType::Stt);
    assert_eq!(stt.len(), 1);
    assert_eq!(stt[0].id, "whisper-tiny-q8");

    let llm = registry.models_by_type(&ModelType::Llm);
    assert_eq!(llm.len(), 1);
    assert!(llm[0].gpu_recommended);
    assert_eq!(llm[0].description.as_deref(), Some("Meeting summary model"));
}

#[test]
fn tc_mdl_001_02_parse_invalid_registry() {
    let bad_toml = "this is not valid toml {{{}}}";
    let result = ModelRegistry::parse(bad_toml);
    assert!(result.is_err(), "Invalid TOML should return error");
}

#[test]
fn tc_mdl_001_03_model_lookup() {
    let toml = r#"
[[model]]
id = "test-model"
name = "Test"
type = "stt"
size_bytes = 100
languages = ["en"]
min_ram_mb = 256
gpu_recommended = false
download_url = "https://example.com/test.bin"
sha256 = "abc"
"#;
    let registry = ModelRegistry::parse(toml).unwrap();
    assert!(registry.get_model("test-model").is_some());
    assert!(registry.get_model("nonexistent").is_none());
}

#[test]
fn tc_mdl_001_04_ram_filtering() {
    let toml = r#"
[[model]]
id = "small"
name = "Small"
type = "stt"
size_bytes = 100
languages = ["en"]
min_ram_mb = 512
gpu_recommended = false
download_url = "https://example.com/small.bin"
sha256 = "a"

[[model]]
id = "large"
name = "Large"
type = "stt"
size_bytes = 1000
languages = ["en"]
min_ram_mb = 8192
gpu_recommended = true
download_url = "https://example.com/large.bin"
sha256 = "b"
"#;
    let registry = ModelRegistry::parse(toml).unwrap();

    let for_2gb = registry.models_for_ram(2048);
    assert_eq!(for_2gb.len(), 1);
    assert_eq!(for_2gb[0].id, "small");

    let for_16gb = registry.models_for_ram(16384);
    assert_eq!(for_16gb.len(), 2);
}

// ── TC-MDL-004-01: SHA-256 무결성 검증 성공 ─────────────────────

#[test]
fn tc_mdl_004_01_sha256_verify_success() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("model.bin");

    let data = b"VoxNote model binary data for testing SHA-256 verification";
    std::fs::write(&path, data).unwrap();

    let hash = integrity::compute_sha256(&path).unwrap();
    assert_eq!(hash.len(), 64); // 32 bytes = 64 hex chars

    let result = integrity::verify_sha256(&path, &hash);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

// ── TC-MDL-004-02: SHA-256 무결성 검증 실패 ─────────────────────

#[test]
fn tc_mdl_004_02_sha256_verify_failure() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("corrupted.bin");
    std::fs::write(&path, b"some data").unwrap();

    let result = integrity::verify_sha256(&path, "0000000000000000000000000000000000000000000000000000000000000000");
    assert!(result.is_err(), "Wrong hash should fail verification");
}

#[test]
fn tc_mdl_004_03_sha256_nonexistent_file() {
    let result = integrity::compute_sha256(std::path::Path::new("/nonexistent/file"));
    assert!(result.is_err());
}

// ── TC-MDL: DiskManager ─────────────────────────────────────────

#[test]
fn tc_mdl_disk_manager_operations() {
    let dir = TempDir::new().unwrap();
    let disk = DiskManager::new(dir.path().to_path_buf(), 1024);

    // 초기 상태
    assert!(!disk.is_downloaded("test-model"));
    assert_eq!(disk.used_bytes().unwrap(), 0);

    // 모델 파일 생성
    let model_path = disk.model_path("test-model");
    std::fs::write(&model_path, b"fake model data").unwrap();

    assert!(disk.is_downloaded("test-model"));
    assert!(disk.used_bytes().unwrap() > 0);

    // 삭제
    disk.delete_model("test-model").unwrap();
    assert!(!disk.is_downloaded("test-model"));
}

#[test]
fn tc_mdl_disk_space_check() {
    let dir = TempDir::new().unwrap();
    let disk = DiskManager::new(dir.path().to_path_buf(), 1); // 1MB 제한

    // 큰 파일로 용량 초과 시뮬레이션
    let model_path = disk.model_path("big-model");
    let mut f = std::fs::File::create(&model_path).unwrap();
    f.write_all(&vec![0u8; 1_100_000]).unwrap(); // 1.1MB

    let result = disk.check_space(500_000); // 0.5MB 추가 요청
    assert!(result.is_err(), "Should fail when exceeding max cache");
}

// ── TC-MDL: Size Display ────────────────────────────────────────

#[test]
fn tc_mdl_size_display() {
    let toml = r#"
[[model]]
id = "tiny"
name = "Tiny"
type = "stt"
size_bytes = 78000000
languages = ["en"]
min_ram_mb = 512
gpu_recommended = false
download_url = "https://example.com/tiny.bin"
sha256 = "x"

[[model]]
id = "large"
name = "Large"
type = "llm"
size_bytes = 4368000000
languages = ["en"]
min_ram_mb = 8192
gpu_recommended = true
download_url = "https://example.com/large.gguf"
sha256 = "y"
"#;
    let reg = ModelRegistry::parse(toml).unwrap();
    assert_eq!(reg.get_model("tiny").unwrap().size_display(), "74 MB");
    assert_eq!(reg.get_model("large").unwrap().size_display(), "4.1 GB");
}
