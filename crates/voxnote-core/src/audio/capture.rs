use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use super::AudioDevice;
use crate::error::AudioError;

/// cpal 기반 마이크 캡처
pub struct CpalCapture {
    device: Device,
    config: StreamConfig,
    stream: Option<Stream>,
}

impl CpalCapture {
    /// 기본 입력 디바이스로 생성
    pub fn new_default() -> Result<Self, AudioError> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(AudioError::NoDevice)?;

        let config = device
            .default_input_config()
            .map_err(|e| AudioError::DeviceNotAvailable(e.to_string()))?;

        info!(
            "Audio device: {}, format: {:?}, rate: {}",
            device.name().unwrap_or_default(),
            config.sample_format(),
            config.sample_rate().0,
        );

        Ok(Self {
            device,
            config: config.into(),
            stream: None,
        })
    }

    /// 특정 디바이스로 생성
    pub fn new_with_device(device_name: &str) -> Result<Self, AudioError> {
        let host = cpal::default_host();
        let device = host
            .input_devices()
            .map_err(|e| AudioError::DeviceNotAvailable(e.to_string()))?
            .find(|d| d.name().map(|n| n == device_name).unwrap_or(false))
            .ok_or_else(|| AudioError::DeviceNotAvailable(device_name.to_string()))?;

        let config = device
            .default_input_config()
            .map_err(|e| AudioError::DeviceNotAvailable(e.to_string()))?;

        Ok(Self {
            device,
            config: config.into(),
            stream: None,
        })
    }

    /// 사용 가능한 입력 디바이스 목록
    pub fn list_devices() -> Result<Vec<AudioDevice>, AudioError> {
        let host = cpal::default_host();
        let default_name = host
            .default_input_device()
            .and_then(|d| d.name().ok());

        let devices = host
            .input_devices()
            .map_err(|e| AudioError::DeviceNotAvailable(e.to_string()))?;

        let mut result = Vec::new();
        for device in devices {
            let name = device.name().unwrap_or_default();
            if let Ok(config) = device.default_input_config() {
                result.push(AudioDevice {
                    is_default: default_name.as_deref() == Some(&name),
                    name,
                    sample_rate: config.sample_rate().0,
                    channels: config.channels(),
                });
            }
        }
        Ok(result)
    }

    /// 캡처 시작 — 콜백으로 f32 샘플 전달
    pub fn start<F>(&mut self, mut callback: F) -> Result<(), AudioError>
    where
        F: FnMut(&[f32], u32, u16) + Send + 'static,
    {
        let sample_rate = self.config.sample_rate.0;
        let channels = self.config.channels;

        let stream = self
            .device
            .build_input_stream(
                &self.config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    callback(data, sample_rate, channels);
                },
                move |err| {
                    error!("Audio stream error: {}", err);
                },
                None,
            )
            .map_err(|e| AudioError::Stream(e.to_string()))?;

        stream
            .play()
            .map_err(|e| AudioError::Stream(e.to_string()))?;

        debug!("Audio capture started: {}Hz, {}ch", sample_rate, channels);
        self.stream = Some(stream);
        Ok(())
    }

    /// 캡처 중지
    pub fn stop(&mut self) {
        if let Some(stream) = self.stream.take() {
            drop(stream);
            debug!("Audio capture stopped");
        }
    }

    /// 현재 캡처 중인지 여부
    pub fn is_capturing(&self) -> bool {
        self.stream.is_some()
    }

    /// 현재 설정의 샘플레이트
    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate.0
    }

    /// 현재 설정의 채널 수
    pub fn channels(&self) -> u16 {
        self.config.channels
    }
}

impl Drop for CpalCapture {
    fn drop(&mut self) {
        self.stop();
    }
}
