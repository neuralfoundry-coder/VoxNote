package app.voxnote.plugins

import android.Manifest
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.MediaRecorder

/**
 * Android AudioRecord 기반 오디오 캡처 플러그인
 *
 * Kotlin에서 AudioRecord로 마이크 PCM 데이터를 캡처하고
 * Rust voxnote-core의 오디오 파이프라인에 전달합니다.
 *
 * Foreground Service로 실행하여 백그라운드 녹음을 지원합니다.
 */
class AudioCapturePlugin {

    private var audioRecord: AudioRecord? = null
    private var isCapturing = false
    private var captureThread: Thread? = null

    companion object {
        private const val SAMPLE_RATE = 16000
        private const val CHANNEL_CONFIG = AudioFormat.CHANNEL_IN_MONO
        private const val AUDIO_FORMAT = AudioFormat.ENCODING_PCM_FLOAT
    }

    fun startCapture(): Boolean {
        val bufferSize = AudioRecord.getMinBufferSize(SAMPLE_RATE, CHANNEL_CONFIG, AUDIO_FORMAT)
        if (bufferSize == AudioRecord.ERROR || bufferSize == AudioRecord.ERROR_BAD_VALUE) {
            return false
        }

        audioRecord = AudioRecord(
            MediaRecorder.AudioSource.MIC,
            SAMPLE_RATE,
            CHANNEL_CONFIG,
            AUDIO_FORMAT,
            bufferSize * 2
        )

        if (audioRecord?.state != AudioRecord.STATE_INITIALIZED) {
            return false
        }

        isCapturing = true
        audioRecord?.startRecording()

        captureThread = Thread {
            val buffer = FloatArray(bufferSize / 4)
            while (isCapturing) {
                val read = audioRecord?.read(buffer, 0, buffer.size, AudioRecord.READ_BLOCKING) ?: 0
                if (read > 0) {
                    // TODO: JNI로 Rust ring buffer에 push
                    // nativePushAudioSamples(buffer, read)
                }
            }
        }
        captureThread?.start()

        return true
    }

    fun stopCapture() {
        isCapturing = false
        captureThread?.join(1000)
        audioRecord?.stop()
        audioRecord?.release()
        audioRecord = null
    }

    // JNI bridge to Rust
    // private external fun nativePushAudioSamples(samples: FloatArray, count: Int)
}
