import AVFoundation
import Tauri

/// iOS AVAudioEngine 기반 오디오 캡처 플러그인
///
/// Swift에서 AVAudioEngine으로 마이크 PCM 데이터를 캡처하고
/// Rust voxnote-core의 오디오 파이프라인에 전달합니다.
class AudioCapturePlugin: Plugin {
    private var audioEngine: AVAudioEngine?
    private var isCapturing = false

    @objc func startCapture(_ call: CAPPluginCall) {
        let audioSession = AVAudioSession.sharedInstance()
        do {
            try audioSession.setCategory(.playAndRecord, mode: .measurement)
            try audioSession.setActive(true)
        } catch {
            call.reject("Failed to configure audio session: \(error)")
            return
        }

        audioEngine = AVAudioEngine()
        guard let engine = audioEngine else {
            call.reject("Failed to create audio engine")
            return
        }

        let inputNode = engine.inputNode
        let format = inputNode.outputFormat(forBus: 0)

        inputNode.installTap(onBus: 0, bufferSize: 4096, format: format) { buffer, time in
            // PCM f32 데이터를 Rust 코어에 전달
            guard let channelData = buffer.floatChannelData else { return }
            let frameLength = Int(buffer.frameLength)
            let samples = Array(UnsafeBufferPointer(start: channelData[0], count: frameLength))
            // TODO: FFI로 Rust ring buffer에 push
        }

        do {
            try engine.start()
            isCapturing = true
            call.resolve(["status": "capturing"])
        } catch {
            call.reject("Failed to start audio engine: \(error)")
        }
    }

    @objc func stopCapture(_ call: CAPPluginCall) {
        audioEngine?.inputNode.removeTap(onBus: 0)
        audioEngine?.stop()
        audioEngine = nil
        isCapturing = false
        call.resolve(["status": "stopped"])
    }
}
