use std::{collections::HashMap, sync::Arc};
// ЗАМЕНА: Используем токио-лок вместо стандартного
use tokio::sync::{broadcast, RwLock};
use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use uuid::Uuid;
use tracing::{error, warn, debug};

use super::dto::WsServerEvent;

const CHANNEL_CAPACITY: usize = 512;

#[derive(Clone, Default, Debug)]
pub struct WsState {
    // ЗАМЕНА: tokio::sync::RwLock
    rooms: Arc<RwLock<HashMap<Uuid, broadcast::Sender<WsServerEvent>>>>,
}

impl WsState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Вернуть sender комнаты (создать если нет)
    pub async fn get_or_create(&self, chat_uuid: Uuid) -> broadcast::Sender<WsServerEvent> {
        // Fast path: читаем асинхронно
        {
            let r = self.rooms.read().await;
            if let Some(tx) = r.get(&chat_uuid) {
                return tx.clone();
            }
        }

        // Slow path: пишем асинхронно
        let mut w = self.rooms.write().await;
        w.entry(chat_uuid)
            .or_insert_with(|| broadcast::channel(CHANNEL_CAPACITY).0)
            .clone()
    }

    /// Разослать событие всем подключённым к чату
    pub async fn broadcast(&self, chat_uuid: Uuid, event: WsServerEvent) {
        let r = self.rooms.read().await;
        if let Some(tx) = r.get(&chat_uuid) {
            // Игнорируем ошибку, если нет подписчиков
            let _ = tx.send(event);
        }
    }

    /// Удалить комнату, если нет активных подписчиков
    pub async fn cleanup(&self, chat_uuid: Uuid) {
        let mut w = self.rooms.write().await;
        if let Some(tx) = w.get(&chat_uuid) {
            if tx.receiver_count() == 0 {
                w.remove(&chat_uuid);
            }
        }
    }
}

pub async fn handle_socket<F, Fut>(
    socket: WebSocket,
    chat_uuid: Uuid,
    user_uuid: Uuid,
    ws_state: WsState,
    on_client_msg: F,
)
where
    F: Fn(Uuid, Uuid, String) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Option<WsServerEvent>> + Send,
{
    let (mut sink, mut stream) = socket.split();

    // ВАЖНО: Вызываем асинхронный метод
    let tx = ws_state.get_or_create(chat_uuid).await;
    let mut rx = tx.subscribe();

    let on_client_msg = Arc::new(on_client_msg);
    let ws_state_for_recv = ws_state.clone();

    // ── Task A: broadcast → клиент ───────────────────────────────
    let mut task_send = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    match serde_json::to_string(&event) {
                        Ok(json) => {
                            if sink.send(Message::Text(json.into())).await.is_err() {
                                break; // Клиент отключился
                            }
                        }
                        Err(e) => {
                            error!("ws serialize error: {e}");
                            // Можно отправить сообщение об ошибке клиенту, если нужно
                            continue;
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!("ws lagged {n} messages");
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    // ── Task B: клиент → обработка → broadcast ───────────────────
    let mut task_recv = tokio::spawn(async move {
        while let Some(Ok(msg)) = stream.next().await {
            match msg {
                Message::Text(text) => {
                    // Вызов обработчика
                    let result = on_client_msg(chat_uuid, user_uuid, text.to_string()).await;

                    if let Some(event) = result {
                        // ВАЖНО: Вызываем асинхронный broadcast
                        ws_state_for_recv.broadcast(chat_uuid, event).await;
                    }
                }
                Message::Close(_) => break,
                _ => {} // Ping, Pong, Binary игнорируем или обрабатываем
            }
        }
    });

    // Ждем завершения любой из задач
    tokio::select! {
        _ = &mut task_send => {
            task_recv.abort();
        },
        _ = &mut task_recv => {
            task_send.abort();
        },
    }

    // Очистка
    ws_state.cleanup(chat_uuid).await;
    debug!("ws disconnected: user={user_uuid} chat={chat_uuid}");
}
