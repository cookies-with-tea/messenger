// src/messenger/ws_echo.rs
use axum::{
    extract::ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::SinkExt;
use std::sync::Arc;
use crate::AppState;

/// Роутер для тестового эхо-эндпоинта
/// Подключается как `.nest("/ws/test", echo_router())` → путь: /ws/test/echo
pub fn echo_router() -> Router<Arc<AppState>> {
    Router::new().route("/echo", get(ws_echo_handler))
}

/// Обработчик апгрейда HTTP → WebSocket
pub async fn ws_echo_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    tracing::info!("🚀 WS upgrade request received at /ws/test/echo");
    ws.on_upgrade(handle_echo_socket)
}

/// Основной цикл обработки WebSocket-сообщений
async fn handle_echo_socket(mut socket: WebSocket) {
    tracing::info!("🔌 Echo WS connected");

    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                tracing::debug!("📥 Received text: {text}");
                // В axum 0.8 Message::Text принимает Utf8Bytes, поэтому .into()
                let reply = Utf8Bytes::from(format!("echo: {text}"));
                if socket.send(Message::Text(reply)).await.is_err() {
                    break;
                }
            }

            Ok(Message::Binary(data)) => {
                tracing::debug!("📥 Received binary ({} bytes)", data.len());
                // Эхо бинарных данных
                if socket.send(Message::Binary(data)).await.is_err() {
                    break;
                }
            }

            Ok(Message::Close(reason)) => {
                tracing::debug!("🔌 Close frame received: {:?}", reason);
                break;
            }

            Ok(Message::Ping(ping)) => {
                // Авто-ответ на Ping (браузеры иногда шлют)
                tracing::debug!("🏓 Ping received");
                if socket.send(Message::Pong(ping)).await.is_err() {
                    break;
                }
            }

            Ok(Message::Pong(_)) => {
                // Игнорируем Pong — это ответ на наш Ping
            }

            Err(e) => {
                tracing::warn!("⚠️ WebSocket error: {e}");
                break;
            }
        }
    }

    tracing::info!("🔌 Echo WS disconnected");
}
