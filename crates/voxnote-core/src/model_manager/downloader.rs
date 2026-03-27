use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

use crate::error::ModelError;
use super::integrity;

/// 다운로드 진행률 콜백
pub type ProgressCallback = Box<dyn Fn(DownloadProgress) + Send + Sync>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DownloadProgress {
    pub model_id: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percentage: f32,
}

/// HTTP Range 기반 재개 가능 모델 다운로더
pub struct ModelDownloader {
    client: reqwest::Client,
}

impl ModelDownloader {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("VoxNote/0.1.0")
            .build()
            .unwrap_or_default();
        Self { client }
    }

    /// 모델 다운로드 (재개 가능)
    ///
    /// - `url`: 다운로드 URL
    /// - `dest`: 저장 경로
    /// - `expected_sha256`: 기대 해시 (검증용)
    /// - `on_progress`: 진행률 콜백
    pub async fn download(
        &self,
        model_id: &str,
        url: &str,
        dest: &Path,
        expected_sha256: &str,
        on_progress: Option<ProgressCallback>,
    ) -> Result<PathBuf, ModelError> {
        // 이미 완료된 파일이 있고 해시가 일치하면 스킵
        if dest.exists() {
            if let Ok(true) = integrity::verify_sha256(dest, expected_sha256) {
                info!("Model already downloaded and verified: {}", model_id);
                return Ok(dest.to_path_buf());
            }
        }

        // 부분 다운로드 파일 확인 (재개)
        let partial_path = dest.with_extension("partial");
        let mut downloaded: u64 = 0;

        if partial_path.exists() {
            downloaded = std::fs::metadata(&partial_path)
                .map(|m| m.len())
                .unwrap_or(0);
            debug!("Resuming download from {} bytes", downloaded);
        }

        // HTTP 요청 (Range 헤더로 재개)
        let mut request = self.client.get(url);
        if downloaded > 0 {
            request = request.header("Range", format!("bytes={}-", downloaded));
        }

        let response = request
            .send()
            .await
            .map_err(|e| ModelError::Download(e.to_string()))?;

        if !response.status().is_success() && response.status().as_u16() != 206 {
            return Err(ModelError::Download(format!(
                "HTTP {}: {}",
                response.status(),
                url
            )));
        }

        let total_bytes = if response.status().as_u16() == 206 {
            // Partial content — total size from Content-Range
            response
                .headers()
                .get("content-range")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split('/').last())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(downloaded + response.content_length().unwrap_or(0))
        } else {
            response.content_length().unwrap_or(0)
        };

        // 부모 디렉토리 생성
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // 파일에 스트리밍 쓰기
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&partial_path)
            .await
            .map_err(|e| ModelError::Io(e))?;

        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| ModelError::Download(e.to_string()))?;
            file.write_all(&chunk)
                .await
                .map_err(|e| ModelError::Io(e))?;
            downloaded += chunk.len() as u64;

            if let Some(ref cb) = on_progress {
                cb(DownloadProgress {
                    model_id: model_id.to_string(),
                    downloaded_bytes: downloaded,
                    total_bytes,
                    percentage: if total_bytes > 0 {
                        (downloaded as f32 / total_bytes as f32) * 100.0
                    } else {
                        0.0
                    },
                });
            }
        }

        file.flush().await.map_err(|e| ModelError::Io(e))?;
        drop(file);

        // SHA-256 검증
        integrity::verify_sha256(&partial_path, expected_sha256).map_err(|e| {
            // 검증 실패 시 부분 파일 삭제
            let _ = std::fs::remove_file(&partial_path);
            e
        })?;

        // 검증 통과 → .partial을 최종 파일로 이동
        std::fs::rename(&partial_path, dest)?;
        info!("Model downloaded and verified: {} → {:?}", model_id, dest);

        Ok(dest.to_path_buf())
    }
}
