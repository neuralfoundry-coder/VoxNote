#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // 모델 검증 모드: 자식 프로세스에서 STT 모델 로드 가능 여부 테스트
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 3 && args[1] == "--validate-stt-model" {
        let path = std::path::PathBuf::from(&args[2]);
        match voxnote_core::stt::whisper::LocalSttProvider::new(path) {
            Ok(_) => std::process::exit(0),
            Err(e) => {
                eprintln!("Model validation failed: {}", e);
                std::process::exit(1);
            }
        }
    }

    voxnote_tauri_lib::run();
}
