use std::path::PathBuf;
use tracing::info;

use crate::error::ModelError;

/// 모델 디스크 관리
pub struct DiskManager {
    models_dir: PathBuf,
    max_cache_mb: u64,
}

impl DiskManager {
    pub fn new(models_dir: PathBuf, max_cache_mb: u64) -> Self {
        Self {
            models_dir,
            max_cache_mb,
        }
    }

    /// 모델 파일 경로
    pub fn model_path(&self, model_id: &str) -> PathBuf {
        self.models_dir.join(model_id)
    }

    /// 모델이 로컬에 존재하는지 확인
    pub fn is_downloaded(&self, model_id: &str) -> bool {
        self.model_path(model_id).exists()
    }

    /// 모델 디렉토리 생성
    pub fn ensure_dir(&self) -> Result<(), ModelError> {
        std::fs::create_dir_all(&self.models_dir)?;
        Ok(())
    }

    /// 다운로드된 모델 목록 (파일명 또는 디렉토리명)
    pub fn list_downloaded(&self) -> Result<Vec<String>, ModelError> {
        if !self.models_dir.exists() {
            return Ok(Vec::new());
        }

        let mut models = Vec::new();
        for entry in std::fs::read_dir(&self.models_dir)? {
            let entry = entry?;
            let ft = entry.file_type()?;
            if let Some(name) = entry.file_name().to_str() {
                // .partial 파일은 제외
                if name.ends_with(".partial") {
                    continue;
                }
                if ft.is_file() || ft.is_dir() {
                    models.push(name.to_string());
                }
            }
        }
        Ok(models)
    }

    /// 현재 사용 중인 디스크 용량 (bytes)
    pub fn used_bytes(&self) -> Result<u64, ModelError> {
        if !self.models_dir.exists() {
            return Ok(0);
        }

        let mut total = 0u64;
        for entry in std::fs::read_dir(&self.models_dir)? {
            let entry = entry?;
            let ft = entry.file_type()?;
            if ft.is_file() {
                total += entry.metadata()?.len();
            } else if ft.is_dir() {
                total += dir_size(&entry.path())?;
            }
        }
        Ok(total)
    }

    /// 모델 삭제 (파일 또는 디렉토리)
    pub fn delete_model(&self, model_id: &str) -> Result<(), ModelError> {
        let path = self.model_path(model_id);
        if path.is_dir() {
            std::fs::remove_dir_all(&path)?;
            info!("Deleted model directory: {}", model_id);
        } else if path.is_file() {
            std::fs::remove_file(&path)?;
            info!("Deleted model file: {}", model_id);
        }
        // partial 파일도 정리
        let partial = path.with_extension("partial");
        if partial.exists() {
            std::fs::remove_file(&partial)?;
        }
        Ok(())
    }

    /// 디스크 여유 공간 확인 (MB)
    pub fn available_space_mb(&self) -> u64 {
        // 플랫폼별 디스크 여유 공간 확인은 복잡하므로 기본값 반환
        // 프로덕션에서는 sysinfo crate 등을 활용
        u64::MAX
    }

    /// 모델이 디렉토리 기반 번들인지 확인
    pub fn is_bundle(&self, model_id: &str) -> bool {
        self.model_path(model_id).is_dir()
    }

    /// 충분한 디스크 공간이 있는지 확인
    pub fn check_space(&self, needed_bytes: u64) -> Result<(), ModelError> {
        let used_bytes = self.used_bytes()?;
        let max_bytes = self.max_cache_mb * 1_048_576;
        if used_bytes + needed_bytes > max_bytes {
            return Err(ModelError::InsufficientDisk {
                need_mb: needed_bytes / 1_048_576,
                available_mb: max_bytes.saturating_sub(used_bytes) / 1_048_576,
            });
        }
        Ok(())
    }
}

/// 디렉토리의 총 크기 계산 (재귀)
fn dir_size(path: &std::path::Path) -> Result<u64, ModelError> {
    let mut total = 0u64;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let ft = entry.file_type()?;
            if ft.is_file() {
                total += entry.metadata()?.len();
            } else if ft.is_dir() {
                total += dir_size(&entry.path())?;
            }
        }
    }
    Ok(total)
}
