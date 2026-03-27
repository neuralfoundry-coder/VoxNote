use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use tracing::{debug, info};

pub fn router() -> Router {
    Router::new()
        .route("/connect", get(ws_upgrade))
        .route("/status", get(sync_status))
}

async fn ws_upgrade(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_websocket)
}

async fn handle_websocket(mut socket: WebSocket) {
    info!("Sync WebSocket connected");

    // CRDT 델타 중계 루프
    // 서버는 암호화된 바이너리 델타만 중계하며, 복호화할 수 없습니다.
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Binary(data)) => {
                debug!("Relaying encrypted delta: {} bytes", data.len());
                // 같은 계정의 다른 디바이스들에게 중계
                // TODO: 룸 관리, 오프라인 버퍼링 (30일 TTL)
                if socket.send(Message::Binary(data)).await.is_err() {
                    break;
                }
            }
            Ok(Message::Ping(data)) => {
                if socket.send(Message::Pong(data)).await.is_err() {
                    break;
                }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }

    info!("Sync WebSocket disconnected");
}

async fn sync_status() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "connected_devices": 0,
        "pending_deltas": 0
    }))
}
