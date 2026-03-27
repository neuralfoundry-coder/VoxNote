// Global shortcut plugin — Phase 1
//
// Tauri의 global-shortcut 플러그인을 활용하여
// 녹음 시작/중지 단축키를 등록합니다.
//
// 실제 단축키 등록은 Tauri 앱 setup에서 수행하며,
// 이 모듈은 단축키 이벤트 핸들러를 제공합니다.

use tracing::info;

/// 기본 단축키 설정
pub const DEFAULT_TOGGLE_RECORDING: &str = "CmdOrCtrl+Shift+R";

pub fn on_toggle_recording() {
    info!("Global shortcut: toggle recording");
    // TODO: recording 상태를 토글하고 UI에 이벤트 전달
}
