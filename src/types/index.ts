// ─── Enums ────────────────────────────────────────────────────────
export type ChatType       = 'direct' | 'group'
export type DeliveryStatus = 'delivered' | 'read'
export type ConnectionStatus = 'connecting' | 'connected' | 'disconnected' | 'error'

// ─── Backend DTOs ─────────────────────────────────────────────────
export interface UserPreviewDTO {
  uuid: string
  first_name: string | null
  second_name: string | null
  avatar: { url: string; alt?: string; title?: string } | null
  is_online: boolean
  last_seen_at: string
}

export interface ChatResponseDTO {
  uuid: string
  name: string | null
  description: string | null
  chat_type: ChatType
  created_by: string
  is_archived: boolean
  created_at: string
  updated_at: string
  last_message_body: string | null
  last_message_at: string | null
  unread_count: number | null
  member_count: number | null
  sender: UserPreviewDTO | null
}

export interface ChatMemberDTO {
  uuid: string
  chat_uuid: string
  user_uuid: string
  is_admin: boolean
  joined_at: string
  left_at: string | null
  first_name: string | null
  last_name: string | null
  avatar: string | null
  is_online: boolean
  last_seen_at: string
}

export interface MessageResponseDTO {
  uuid: string
  chat_uuid: string
  sender_uuid: string
  reply_to_uuid: string | null
  body: string
  is_edited: boolean
  is_deleted: boolean
  created_at: string
  updated_at: string
  sender: UserPreviewDTO | null
  delivered_count: number | null
  read_count: number | null
  my_status: DeliveryStatus | null
  reply_body_preview: string | null
}

// ─── API wrappers ─────────────────────────────────────────────────
export interface ApiResponse<T> {
  data: T | null
  errors: Record<string, string> | null
  messages: string[] | null
}

export interface ApiPaginationData<T> {
  items: T[]
  pagination: { page: number; total: number | null; total_pages: number | null; limit: number | null }
}

export interface ApiResponseWithPagination<T> {
  data: ApiPaginationData<T> | null
  errors: Record<string, string> | null
  messages: string[] | null
}

// ─── WebSocket (сервер → клиент) ──────────────────────────────────
export type WsServerEvent =
  | { event: 'new_message';     payload: MessageResponseDTO }
  | { event: 'message_edited';  payload: MessageResponseDTO }
  | { event: 'message_deleted'; payload: { uuid: string; chat_uuid: string } }
  | { event: 'status_updated';  payload: { chat_uuid: string; message_uuid: string; user_uuid: string; status: DeliveryStatus } }
  | { event: 'typing';          payload: { chat_uuid: string; user_uuid: string; is_typing: boolean } }
  | { event: 'member_joined';   payload: ChatMemberDTO }
  | { event: 'member_left';     payload: { chat_uuid: string; user_uuid: string } }
  | { event: 'user_status_changed'; payload: { user_uuid: string; is_online: boolean; last_seen_at: string } }
  | { event: 'error';           payload: { message: string } }
  | { event: 'pong' }

// ─── WebSocket (клиент → сервер) ──────────────────────────────────
export type WsClientAction =
  | { action: 'send_message';   payload: { body: string; reply_to_uuid: string | null } }
  | { action: 'edit_message';   payload: { uuid: string; body: string } }
  | { action: 'delete_message'; payload: { uuid: string } }
  | { action: 'mark_delivered'; payload: { chat_uuid: string } }
  | { action: 'mark_read';      payload: { chat_uuid: string } }
  | { action: 'typing';         payload: { is_typing: boolean } }
