mod commands;
mod state;

use tauri::Manager;

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".parse().unwrap()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_state = state::AppState::new()?;
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Recording
            commands::recording::start_recording,
            commands::recording::stop_recording,
            commands::recording::pause_recording,
            // Notes
            commands::note::list_notes,
            commands::note::get_note,
            commands::note::create_note,
            commands::note::update_note,
            commands::note::delete_note,
            commands::note::search_notes,
            // Models
            commands::model::list_models,
            commands::model::download_model,
            commands::model::delete_model,
            // Settings
            commands::settings::get_settings,
            commands::settings::update_settings,
            // LLM (Phase 2)
            commands::llm::generate_summary,
            commands::llm::ask_voxnote,
            // Provider (Phase 2)
            commands::provider::get_provider_config,
            commands::provider::set_provider_config,
            commands::provider::test_provider,
            commands::provider::list_available_providers,
            // Export (Phase 2)
            commands::export::export_note,
        ])
        .run(tauri::generate_context!())
        .expect("error while running VoxNote");
}
