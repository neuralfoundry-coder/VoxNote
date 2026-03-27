use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, warn};

/// WebSocket 동기화 클라이언트
///
/// - 자동 재연결 (exponential backoff)
/// - 30초 heartbeat
/// - E2EE 암호화 델타 전송/수신
pub struct SyncClient {
    server_url: String,
    auth_token: String,
    is_connected: bool,
}

/// 동기화 이벤트
#[derive(Debug, Clone)]
pub enum SyncEvent {
    Connected,
    Disconnected,
    DeltaReceived { data: Vec<u8> },
    Error { message: String },
}

impl SyncClient {
    pub fn new(server_url: &str, auth_token: &str) -> Self {
        Self {
            server_url: server_url.to_string(),
            auth_token: auth_token.to_string(),
            is_connected: false,
        }
    }

    /// WebSocket 연결 및 메시지 루프
    pub async fn connect(
        &mut self,
        event_tx: mpsc::UnboundedSender<SyncEvent>,
        mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
    ) {
        let mut retry_count = 0u32;
        let max_retries = 10;

        loop {
            if *shutdown_rx.borrow() {
                break;
            }

            let url = format!("{}/api/v1/sync/connect?token={}", self.server_url, self.auth_token);

            match tokio_tungstenite::connect_async(&url).await {
                Ok((ws_stream, _)) => {
                    info!("Sync connected to {}", self.server_url);
                    self.is_connected = true;
                    retry_count = 0;
                    let _ = event_tx.send(SyncEvent::Connected);

                    use futures_util::{SinkExt, StreamExt};
                    let (mut write, mut read) = ws_stream.split();

                    // heartbeat 타이머
                    let heartbeat_interval = tokio::time::interval(
                        tokio::time::Duration::from_secs(30),
                    );
                    tokio::pin!(heartbeat_interval);

                    loop {
                        tokio::select! {
                            _ = shutdown_rx.changed() => {
                                if *shutdown_rx.borrow() {
                                    let _ = write.send(Message::Close(None)).await;
                                    break;
                                }
                            }
                            _ = heartbeat_interval.tick() => {
                                if write.send(Message::Ping(vec![])).await.is_err() {
                                    break;
                                }
                            }
                            msg = read.next() => {
                                match msg {
                                    Some(Ok(Message::Binary(data))) => {
                                        let _ = event_tx.send(SyncEvent::DeltaReceived { data });
                                    }
                                    Some(Ok(Message::Close(_))) | None => {
                                        info!("Sync connection closed");
                                        break;
                                    }
                                    Some(Err(e)) => {
                                        warn!("Sync receive error: {}", e);
                                        break;
                                    }
                                    _ => {} // Ping/Pong/Text handled by library
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Sync connection failed: {}", e);
                    let _ = event_tx.send(SyncEvent::Error {
                        message: e.to_string(),
                    });
                }
            }

            self.is_connected = false;
            let _ = event_tx.send(SyncEvent::Disconnected);

            // Exponential backoff
            retry_count += 1;
            if retry_count > max_retries {
                error!("Max sync retries reached, giving up");
                break;
            }
            let delay = std::time::Duration::from_secs(2u64.pow(retry_count.min(6)));
            debug!("Reconnecting in {:?} (attempt {})", delay, retry_count);
            tokio::time::sleep(delay).await;
        }
    }

    /// 암호화 델타 전송
    pub async fn send_delta(&self, _encrypted_data: Vec<u8>) -> Result<(), String> {
        if !self.is_connected {
            return Err("Not connected".to_string());
        }
        // TODO: ws_stream.send() 호출
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected
    }
}
