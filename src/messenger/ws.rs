use std::{collections::HashMap, sync::Arc};
use tokio::sync::{broadcast, RwLock};
use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use uuid::Uuid;
use tracing::{error, warn, debug, info};
use sqlx::{Pool, Postgres};

use super::dto::WsServerEvent;

const CHANNEL_CAPACITY: usize = 512;

// ═══════════════════════════════════════════════════════════════
// Per-Chat WsState (оставляем для совместимости с REST broadcast)
// ═══════════════════════════════════════════════════════════════

#[derive(Clone, Default, Debug)]
pub struct WsState {
    rooms: Arc<RwLock<HashMap<Uuid, broadcast::Sender<WsServerEvent>>>>,
}

impl WsState {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get_or_create(&self, chat_uuid: Uuid) -> broadcast::Sender<WsServerEvent> {
        {
            let r = self.rooms.read().await;
            if let Some(tx) = r.get(&chat_uuid) {
                return tx.clone();
            }
        }
        let mut w = self.rooms.write().await;
        w.entry(chat_uuid)
            .or_insert_with(|| broadcast::channel(CHANNEL_CAPACITY).0)
            .clone()
    }

    /// Разослать событие всем подключённым к чату (синхронно — без await)
    pub fn broadcast(&self, chat_uuid: Uuid, event: WsServerEvent) {
        let rooms = self.rooms.clone();
        let event_clone = event.clone();
        tokio::spawn(async move {
            let r = rooms.read().await;
            if let Some(tx) = r.get(&chat_uuid) {
                let _ = tx.send(event_clone);
            }
        });
    }

    pub async fn cleanup(&self, chat_uuid: Uuid) {
        let mut w = self.rooms.write().await;
        if let Some(tx) = w.get(&chat_uuid) {
            if tx.receiver_count() == 0 {
                w.remove(&chat_uuid);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Per-User UserWsState — глобальный канал на пользователя
// ═══════════════════════════════════════════════════════════════

#[derive(Clone, Default, Debug)]
pub struct UserWsState {
    users: Arc<RwLock<HashMap<Uuid, broadcast::Sender<WsServerEvent>>>>,
}

impl UserWsState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Вернуть (или создать) sender для пользователя
    pub async fn get_or_create(&self, user_uuid: Uuid) -> broadcast::Sender<WsServerEvent> {
        {
            let r = self.users.read().await;
            if let Some(tx) = r.get(&user_uuid) {
                return tx.clone();
            }
        }
        let mut w = self.users.write().await;
        w.entry(user_uuid)
            .or_insert_with(|| broadcast::channel(CHANNEL_CAPACITY).0)
            .clone()
    }

    /// Разослать событие конкретному пользователю
    pub async fn send_to_user(&self, user_uuid: Uuid, event: WsServerEvent) {
        let r = self.users.read().await;
        if let Some(tx) = r.get(&user_uuid) {
            let _ = tx.send(event);
        }
    }

    /// Разослать событие списку пользователей (всем участникам чата)
    pub async fn broadcast_to_users(&self, user_uuids: &[Uuid], event: WsServerEvent) {
        let r = self.users.read().await;
        for uuid in user_uuids {
            if let Some(tx) = r.get(uuid) {
                let _ = tx.send(event.clone());
            }
        }
    }

    /// Удалить пользователя, если нет активных подписчиков
    pub async fn cleanup(&self, user_uuid: Uuid) {
        let mut w = self.users.write().await;
        if let Some(tx) = w.get(&user_uuid) {
            if tx.receiver_count() == 0 {
                w.remove(&user_uuid);
            }
        }
    }

    /// Проверить, в сети ли пользователь
    pub async fn is_online(&self, user_uuid: Uuid) -> bool {
        let r = self.users.read().await;
        if let Some(tx) = r.get(&user_uuid) {
            tx.receiver_count() > 0
        } else {
            false
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Per-Chat socket handler (используется из старого ws_router — для совместимости)
// ═══════════════════════════════════════════════════════════════

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

    let tx = ws_state.get_or_create(chat_uuid).await;
    let mut rx = tx.subscribe();

    let on_client_msg = Arc::new(on_client_msg);
    let ws_state_for_recv = ws_state.clone();

    let mut task_send = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    match serde_json::to_string(&event) {
                        Ok(json) => {
                            if sink.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            error!("ws serialize error: {e}");
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

    let mut task_recv = tokio::spawn(async move {
        while let Some(Ok(msg)) = stream.next().await {
            match msg {
                Message::Text(text) => {
                    let result = on_client_msg(chat_uuid, user_uuid, text.to_string()).await;
                    if let Some(event) = result {
                        ws_state_for_recv.broadcast(chat_uuid, event);
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = &mut task_send => { task_recv.abort(); },
        _ = &mut task_recv => { task_send.abort(); },
    }

    ws_state.cleanup(chat_uuid).await;
    debug!("ws disconnected: user={user_uuid} chat={chat_uuid}");
}

// ═══════════════════════════════════════════════════════════════
// Per-User socket handler — глобальный WS
// ═══════════════════════════════════════════════════════════════

pub async fn handle_user_socket<F, Fut>(
    socket: WebSocket,
    user_uuid: Uuid,
    user_ws_state: UserWsState,
    pool: Pool<Postgres>,
    on_client_msg: F,
)
where
    F: Fn(Uuid, String) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send,
{
    let (mut sink, mut stream) = socket.split();

    // ─── 1. Connect logic: Store online status ───
    // Optimize: only update if user was offline to reduce DB load
    let _ = sqlx::query("UPDATE guest_user SET is_online = TRUE WHERE uuid = $1 AND is_online = FALSE")
        .bind(user_uuid)
        .execute(&pool)
        .await;

    // Broadcast status change to chat participants
    let user_ws_state_for_broadcast = user_ws_state.clone();
    let pool_for_broadcast = pool.clone();
    tokio::spawn(async move {
        let participants = sqlx::query_scalar::<_, Uuid>(
            "SELECT DISTINCT cm2.user_uuid 
             FROM chat_member cm1
             JOIN chat_member cm2 ON cm2.chat_uuid = cm1.chat_uuid
             WHERE cm1.user_uuid = $1 AND cm1.left_at IS NULL AND cm2.left_at IS NULL"
        )
        .bind(user_uuid)
        .fetch_all(&pool_for_broadcast)
        .await
        .unwrap_or_default();

        user_ws_state_for_broadcast.broadcast_to_users(
            &participants, 
            WsServerEvent::UserStatusChanged { 
                user_uuid, 
                is_online: true, 
                last_seen_at: Some(chrono::Utc::now()) 
            }
        ).await;
    });

    // Подписываемся на канал данного пользователя
    let tx = user_ws_state.get_or_create(user_uuid).await;
    let mut rx = tx.subscribe();

    let on_client_msg = Arc::new(on_client_msg);

    // Task A: user_ws broadcast → клиент
    let mut task_send = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    match serde_json::to_string(&event) {
                        Ok(json) => {
                            if sink.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            error!("user_ws serialize error: {e}");
                            continue;
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!("user_ws lagged {n} messages for user={user_uuid}");
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    // Task B: клиент → обработчик (handler сам делает broadcast нужным пользователям)
    let mut task_recv = tokio::spawn(async move {
        while let Some(Ok(msg)) = stream.next().await {
            match msg {
                Message::Text(text) => {
                    on_client_msg(user_uuid, text.to_string()).await;
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = &mut task_send => { task_recv.abort(); },
        _ = &mut task_recv => { task_send.abort(); },
    }

    user_ws_state.cleanup(user_uuid).await;
    
    // ─── 2. Disconnect logic: Store offline status ───
    // Only set to FALSE if NO MORE receivers (all tabs closed)
    if !user_ws_state.is_online(user_uuid).await {
        let now = chrono::Utc::now();
        let _ = sqlx::query("UPDATE guest_user SET is_online = FALSE, last_seen_at = $2 WHERE uuid = $1")
            .bind(user_uuid)
            .bind(now)
            .execute(&pool)
            .await;

        // Broadcast disconnect
        let user_ws_state_for_disconnect = user_ws_state.clone();
        tokio::spawn(async move {
            let participants = sqlx::query_scalar::<_, Uuid>(
                "SELECT DISTINCT cm2.user_uuid 
                 FROM chat_member cm1
                 JOIN chat_member cm2 ON cm2.chat_uuid = cm1.chat_uuid
                 WHERE cm1.user_uuid = $1 AND cm1.left_at IS NULL AND cm2.left_at IS NULL"
            )
            .bind(user_uuid)
            .fetch_all(&pool)
            .await
            .unwrap_or_default();

            user_ws_state_for_disconnect.broadcast_to_users(
                &participants, 
                WsServerEvent::UserStatusChanged { 
                    user_uuid, 
                    is_online: false, 
                    last_seen_at: Some(now) 
                }
            ).await;
        });
    }

    debug!("user_ws disconnected: user={user_uuid}");
}
