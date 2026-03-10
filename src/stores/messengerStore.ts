import { defineStore } from "pinia";
import { ref, computed, watch } from "vue";
import { useWebSocket } from "@vueuse/core";
import type { ChatResponseDTO, ChatMemberDTO, MessageResponseDTO, UserResponseDTO, DeliveryStatus, WsServerEvent } from "@/types";
import { chatApi, messageApi, mediaApi, userApi, wsUserUrl, tokenStore } from "@/api";
import { useRouter } from "vue-router";

// UUID текущего пользователя — берётся из JWT payload
function parseCurrentUserId(): string {
	try {
		const token = localStorage.getItem("access_token") ?? "";
		if (!token) return "";
		const parts = token.split(".");
		if (parts.length < 2) return "";
		const payload = JSON.parse(atob(parts[1]));
		return payload.sub ?? payload.uuid ?? payload.id ?? "";
	} catch {
		return "";
	}
}

export const useMessengerStore = defineStore("messenger", () => {
	const router = useRouter();

	// ─── State ────────────────────────────────────────────────────────
	const currentUserId = ref(parseCurrentUserId());

	const chats = ref<ChatResponseDTO[]>([]);
	const currentUserProfile = ref<UserResponseDTO | null>(null);
	const chatsLoading = ref(false);

	const messages = ref<Map<string, MessageResponseDTO[]>>(new Map());
	const messagesLoading = ref(false);

	const members = ref<Map<string, ChatMemberDTO[]>>(new Map());

	const activeChatId = ref<string | null>(null);

	// uuid → Set<user_uuid> кто сейчас печатает
	const typingMap = ref<Map<string, Set<string>>>(new Map());

	// Edition & Reply State
	const replyingToMessage = ref<MessageResponseDTO | null>(null);
	const editingMessage = ref<MessageResponseDTO | null>(null);

	// Track which chats have had their full history fetched
	const loadedChats = ref<Set<string>>(new Set());

	// ─── Global WebSocket (per-user) ──────────────────────────────────
	// Подключается один раз при логине, получает события по всем чатам.
	// immediate: false — открываем вручную через initWs()
	const wsEndpoint = computed(() => wsUserUrl());
	const { status, data, send, open, close } = useWebSocket(wsEndpoint, {
		immediate: false,
		autoReconnect: {
			delay: 2000,
			retries: 10,
		},
		heartbeat: {
			message: JSON.stringify({ action: "ping" }),
			interval: 30000,
		},
	});

	// ─── Computed ─────────────────────────────────────────────────────
	const activeChat = computed(() => chats.value.find((c) => c.uuid === activeChatId.value) ?? null);

	const activeMessages = computed(() => (activeChatId.value ? (messages.value.get(activeChatId.value) ?? []) : []));

	function typingUsersFor(chatUuid: string): string[] {
		return Array.from(typingMap.value.get(chatUuid) ?? []);
	}

	// ─── Helper: display name for a member ───────────────────────────
	function memberName(m: ChatMemberDTO): string {
		return [m.first_name, m.last_name].filter(Boolean).join(" ") || m.user_uuid;
	}

// ─── STORES ───────────────────────────────────────────────────────

	function sendRawWsMessage(msg: Record<string, any>) {
		if (status.value === "OPEN") {
			send(JSON.stringify(msg));
		} else {
			console.warn("[WS] Cannot send raw message, socket not OPEN", msg);
		}
	}

	function handleWsMessage(event: string) {
		console.debug("[WS:Incoming]", event);
		try {
			const ev: WsServerEvent = JSON.parse(event);
			console.debug("[WS:ParsedEvent]", ev.event, ev);

			switch (ev.event) {
				case "new_message": {
					const msg = ev.payload;
					_appendMessage(msg);
					_bumpChat(msg.chat_uuid, msg.body, msg.created_at);
					
					// Авто-подтверждение получения/прочтения (только если не мы отправители)
					if (msg.sender_uuid !== currentUserId.value) {
						if (msg.chat_uuid === activeChatId.value) {
							markRead(msg.chat_uuid);
						} else {
							markDelivered(msg.chat_uuid);
						}
					}
					break;
				}

				case "message_edited":
					_replaceMessage(ev.payload);
					break;

				case "message_deleted":
					_removeMessage(ev.payload.chat_uuid, ev.payload.uuid);
					break;

				case "status_updated":
					_applyStatus(ev.payload.chat_uuid, ev.payload.user_uuid, ev.payload.status);
					break;

				case "typing": {
					const { chat_uuid, user_uuid, is_typing } = ev.payload;
					if (user_uuid === currentUserId.value) return;
					const set = typingMap.value.get(chat_uuid) ?? new Set();
					is_typing ? set.add(user_uuid) : set.delete(user_uuid);
					typingMap.value.set(chat_uuid, new Set(set));
					break;
				}

				case "user_status_changed": {
					const { user_uuid, is_online, last_seen_at } = ev.payload;
					_updateUserStatus(user_uuid, is_online, last_seen_at);
					break;
				}

				case "message_reaction_updated": {
					const { chat_uuid, message_uuid, user_uuid, emoji, is_added } = ev.payload;
					_updateReaction(chat_uuid, message_uuid, user_uuid, emoji, is_added);
					break;
				}

				// ─── WebRTC Signaling ───
				case "call_offer": {
					import("./callStore").then(({ useCallStore }) => {
						// @ts-ignore
						useCallStore().receiveOffer(ev.payload.chat_uuid, ev.payload.caller_uuid, ev.payload.sdp);
					});
					break;
				}
				case "call_answer": {
					import("./callStore").then(({ useCallStore }) => {
						// @ts-ignore
						useCallStore().receiveAnswer(ev.payload.chat_uuid, ev.payload.responder_uuid, ev.payload.sdp);
					});
					break;
				}
				case "ice_candidate": {
					import("./callStore").then(({ useCallStore }) => {
						// @ts-ignore
						useCallStore().receiveIceCandidate(ev.payload.chat_uuid, ev.payload.candidate, ev.payload.sdp_mid, ev.payload.sdp_m_line_index);
					});
					break;
				}
				case "call_reject": {
					import("./callStore").then(({ useCallStore }) => {
						// @ts-ignore
						useCallStore().handleRemoteCallReject(ev.payload.chat_uuid);
					});
					break;
				}
				case "call_end": {
					import("./callStore").then(({ useCallStore }) => {
						// @ts-ignore
						useCallStore().handleRemoteCallEnd(ev.payload.chat_uuid);
					});
					break;
				}

				case "error":
					console.error("[WS Error]", ev.payload.message);
					break;

				case "pong":
					// Игнорируем пинг-понг
					break;

				default:
					console.warn("[WS] Unknown event:", (ev as any).event);
			}
		} catch (e) {
			console.error("[WS] Failed to parse message:", e);
		}
	}

	// Подписываемся на входящие сообщения глобального WS
	watch(data, (newData) => {
		if (newData) {
			handleWsMessage(newData);
		}
	});

	// Initial loads
	if (tokenStore.isLoggedIn()) {
		fetchCurrentUser();
	}

	// ─── Init WS (вызывать после логина) ─────────────────────────────
	function initWs() {
		const token = localStorage.getItem("access_token");
		if (!token) {
			console.warn("[WS] No token, cannot init WS");
			return;
		}
		if (status.value === "OPEN" || status.value === "CONNECTING") return;
		console.debug("[WS] Opening connection to:", wsEndpoint.value);
		open();
		fetchCurrentUser();
	}

	// ─── Load chats ───────────────────────────────────────────────────
	async function fetchChats() {
		chatsLoading.value = true;
		try {
			const res = await chatApi.list();
			chats.value = res.data?.items ?? [];
		} finally {
			chatsLoading.value = false;
		}
	}

	async function fetchCurrentUser() {
		if (!currentUserId.value) return;
		try {
			const res = await userApi.get(currentUserId.value);
			currentUserProfile.value = res.data;
		} catch (e) {
			console.error("[Store] Failed to fetch current user profile:", e);
		}
	}

	// ─── Select & open chat ───────────────────────────────────────────
	// WS НЕ переподключается — он глобальный и уже открыт
	async function selectChat(chatUuid: string) {
		if (activeChatId.value === chatUuid) return;

		activeChatId.value = chatUuid;

		// Reset unread locally
		const chat = chats.value.find((c) => c.uuid === chatUuid);
		if (chat) chat.unread_count = 0;

		// Load messages if not cached or fully loaded
		if (!loadedChats.value.has(chatUuid)) {
			await fetchMessages(chatUuid);
		}

		// Mark as read via WS and API
		markRead(chatUuid);
		messageApi.markRead(chatUuid).catch(() => { });
	}

	// ─── Load messages ────────────────────────────────────────────────
	async function fetchMessages(chatUuid: string, beforeUuid?: string) {
		if (messagesLoading.value) return;
		messagesLoading.value = true;
		try {
			const res = await messageApi.list(chatUuid, { limit: 50, before_uuid: beforeUuid });
			const incoming = res.data?.items ?? [];
			if (beforeUuid) {
				const existing = messages.value.get(chatUuid) ?? [];
				messages.value.set(chatUuid, [...incoming, ...existing]);
			} else {
				messages.value.set(chatUuid, incoming);
				loadedChats.value.add(chatUuid);
			}
		} finally {
			messagesLoading.value = false;
		}
	}

	// ─── Send message via global WS ───────────────────────────────────
	async function sendMessage(body: string, replyToUuid?: string) {
		const chatUuid = activeChatId.value;
		if (!chatUuid || !body.trim()) return;

		if (status.value !== "OPEN") {
			console.warn("[WS] Not connected, trying to connect and retry...");
			initWs();
			
			// Ждем открытия сокета короткое время и пробуем снова (простое решение)
			let attempts = 0;
			const checkOpen = setInterval(() => {
				attempts++;
				if (status.value === "OPEN") {
					clearInterval(checkOpen);
					_performSend(chatUuid, body, replyToUuid);
				} else if (attempts > 20) {
					clearInterval(checkOpen);
					console.error("[WS] Failed to connect for sending message");
				}
			}, 200);
			return;
		}

		_performSend(chatUuid, body, replyToUuid);
	}

	function _performSend(chatUuid: string, body: string, replyToUuid?: string, mediaUuid?: string) {
		console.debug("[WS:Send]", { chatUuid, body, mediaUuid, replyToUuid });
		send(JSON.stringify({
			action: "send_message",
			payload: {
				chat_uuid: chatUuid,
				body: body.trim(),
				reply_to_uuid: replyToUuid ?? replyingToMessage.value?.uuid ?? null,
				media_uuid: mediaUuid ?? null,
			},
		}));
		// Clear local state after send
		replyingToMessage.value = null;
		editingMessage.value = null;
	}

	async function sendVoiceMessage(blob: Blob, replyToUuid?: string) {
		const chatUuid = activeChatId.value;
		if (!chatUuid) return;

		try {
			// 1. Upload
			const res = await mediaApi.upload(blob, "voice_message.webm");
			if (res.data?.uuid) {
				// 2. Send via WS
				_performSend(chatUuid, "[Voice Message]", replyToUuid ?? replyingToMessage.value?.uuid, res.data.uuid);
			}
		} catch (e) {
			console.error("[VoiceMessage] Failed to upload/send:", e);
		}
	}

	async function sendFileMessage(file: File) {
		const chatUuid = activeChatId.value;
		if (!chatUuid) return;

		try {
			const res = await mediaApi.upload(file, file.name);
			if (res.data?.uuid) {
				const label = file.type.startsWith('image/') ? '[Image]' : `[File: ${file.name}]`;
				_performSend(chatUuid, label, replyingToMessage.value?.uuid, res.data.uuid);
			}
		} catch (e) {
			console.error("[FileSend] Failed:", e);
		}
	}

	// ─── Edit message ─────────────────────────────────────────────────
	async function editMessage(msgUuid: string, body: string) {
		const chatUuid = activeChatId.value;
		if (!chatUuid) return;

		send(JSON.stringify({
			action: "edit_message",
			payload: { chat_uuid: chatUuid, uuid: msgUuid, body },
		}));
	}

	// ─── Delete message ───────────────────────────────────────────────
	async function deleteMessage(msgUuid: string) {
		const chatUuid = activeChatId.value;
		if (!chatUuid) return;

		send(JSON.stringify({
			action: "delete_message",
			payload: { uuid: msgUuid },
		}));
	}

	function sendReaction(msgUuid: string, emoji: string) {
		const chatUuid = activeChatId.value;
		if (!chatUuid) return;

		send(JSON.stringify({
			action: "react_to_message",
			payload: { chat_uuid: chatUuid, message_uuid: msgUuid, emoji },
		}));
	}

	// ─── Typing events ────────────────────────────────────────────────
	function sendTyping(isTyping: boolean) {
		const chatUuid = activeChatId.value;
		if (!chatUuid || status.value !== "OPEN") return;

		send(JSON.stringify({
			action: "typing",
			payload: { chat_uuid: chatUuid, is_typing: isTyping },
		}));
	}

	function markRead(chatUuid: string) {
		if (status.value !== "OPEN") return;
		send(JSON.stringify({
			action: "mark_read",
			payload: { chat_uuid: chatUuid },
		}));
		// Сбрасываем счетчик локально
		const chat = chats.value.find(c => c.uuid === chatUuid);
		if (chat) chat.unread_count = 0;
	}

	function markDelivered(chatUuid: string) {
		if (status.value !== "OPEN") return;
		send(JSON.stringify({
			action: "mark_delivered",
			payload: { chat_uuid: chatUuid },
		}));
	}

	// ─── Internal message helpers ─────────────────────────────────────
	function _appendMessage(msg: MessageResponseDTO) {
		const list = messages.value.get(msg.chat_uuid) ?? [];
		if (list.find((m) => m.uuid === msg.uuid)) return;
		messages.value.set(msg.chat_uuid, [...list, msg]);

		if (activeChatId.value !== msg.chat_uuid && msg.sender_uuid !== currentUserId.value) {
			const chat = chats.value.find((c) => c.uuid === msg.chat_uuid);
			if (chat) chat.unread_count = (chat.unread_count ?? 0) + 1;
		}
	}

	function _replaceMessage(msg: MessageResponseDTO) {
		const list = messages.value.get(msg.chat_uuid);
		if (!list) return;
		const idx = list.findIndex((m) => m.uuid === msg.uuid);
		if (idx !== -1) {
			list.splice(idx, 1, msg);
			messages.value.set(msg.chat_uuid, [...list]);
		}
	}

	function _updateUserStatus(userUuid: string, isOnline: boolean, lastSeenAt: string) {
		// 1. Обновляем в списке чатов (если это прямой чат с этим пользователем)
		chats.value.forEach((chat) => {
			if (chat.sender && chat.sender.uuid === userUuid) {
				chat.sender.is_online = isOnline;
				chat.sender.last_seen_at = lastSeenAt;
			}
		});

		// 2. Обновляем в списках участников (если кешированы)
		members.value.forEach((memberList) => {
			memberList.forEach((member) => {
				if (member.user_uuid === userUuid) {
					member.is_online = isOnline;
					member.last_seen_at = lastSeenAt;
				}
			});
		});

		// 3. Обновляем в сообщениях (опционально, так как там sender может быть закеширован)
		messages.value.forEach((msgList) => {
			msgList.forEach((msg) => {
				if (msg.sender && msg.sender.uuid === userUuid) {
					msg.sender.is_online = isOnline;
					msg.sender.last_seen_at = lastSeenAt;
				}
			});
		});
	}

	function _removeMessage(chatUuid: string, msgUuid: string) {
		const list = messages.value.get(chatUuid);
		if (!list) return;
		messages.value.set(
			chatUuid,
			list.filter((m) => m.uuid !== msgUuid),
		);
	}

	function _updateReaction(chatUuid: string, msgUuid: string, userUuid: string, emoji: string, isAdded: boolean) {
		const list = messages.value.get(chatUuid);
		if (!list) return;

		const msg = list.find((m) => m.uuid === msgUuid);
		if (!msg) return;

		if (!msg.reactions) msg.reactions = [];

		if (isAdded) {
			// Добавляем если нет
			if (!msg.reactions.find((r) => r.user_uuid === userUuid && r.emoji === emoji)) {
				msg.reactions.push({ user_uuid: userUuid, emoji });
			}
		} else {
			// Удаляем
			msg.reactions = msg.reactions.filter((r) => !(r.user_uuid === userUuid && r.emoji === emoji));
		}
		
		messages.value.set(chatUuid, [...list]);
	}

	async function _bumpChat(chatUuid: string, lastBody: string, lastAt: string) {
		console.debug("[WS:BumpChat]", { chatUuid, lastBody });
		let chat = chats.value.find((c) => c.uuid === chatUuid);
		
		if (!chat) {
			console.debug("[WS:BumpChat] Chat not found locally, fetching...", chatUuid);
			// Если чата нет, пробуем загрузить его (например, новый диалог)
			try {
				const res = await chatApi.get(chatUuid);
				if (res.data) {
					console.debug("[WS:BumpChat] Fetched new chat:", res.data.uuid);
					chats.value.unshift(res.data);
					chat = res.data;
				}
			} catch (err) {
				console.error("[WS:BumpChat] Failed to fetch new chat details:", err);
				return;
			}
		}

		if (chat) {
			chat.last_message_body = lastBody;
			chat.last_message_at = lastAt;
			const idx = chats.value.indexOf(chat);
			if (idx > 0) {
				chats.value.splice(idx, 1);
				chats.value.unshift(chat);
			}
		}
	}

	function _applyStatus(chatUuid: string, userUuid: string, statusVal: DeliveryStatus) {
		const list = messages.value.get(chatUuid);
		if (!list) return;

		// 1. Если это МЫ прочитали/получили (userUuid === currentUserId)
		// Обновляем my_status для всех чужих сообщений
		if (userUuid === currentUserId.value) {
			list.forEach((m) => {
				if (m.sender_uuid !== currentUserId.value) {
					if (statusVal === "read") m.my_status = "read";
					else if (statusVal === "delivered" && m.my_status !== "read") m.my_status = "delivered";
				}
			});
		} 
		// 2. Если КТО-ТО ДРУГОЙ прочитал (userUuid !== currentUserId)
		// Обновляем counts для НАШИХ сообщений (упрощенно: считаем что прочитал всё)
		else {
			list.forEach((m) => {
				if (m.sender_uuid === currentUserId.value) {
					if (statusVal === "read") m.read_count = (m.read_count ?? 0) + 1;
					else if (statusVal === "delivered") m.delivered_count = (m.delivered_count ?? 0) + 1;
				}
			});
		}
	}

	function logout() {
		close(); // Close WebSocket
		tokenStore.clear();

		// Reset state
		currentUserId.value = "";
		chats.value = [];
		messages.value.clear();
		members.value.clear();
		activeChatId.value = null;
		typingMap.value.clear();
		loadedChats.value.clear();

		router.push("/login");
	}

	return {
		// state
		currentUserId,
		currentUserProfile,
		chats,
		chatsLoading,
		messages,
		messagesLoading,
		members,
		activeChatId,
		wsStatus: status,
		replyingToMessage,
		editingMessage,
		// computed
		activeChat,
		activeMessages,
		typingUsersFor,
		memberName,
		// methods
		initWs,
		fetchChats,
		fetchMessages,
		fetchCurrentUser,
		selectChat,
		sendMessage,
		sendVoiceMessage,
		sendFileMessage,
		editMessage,
		deleteMessage,
		sendTyping,
		markRead,
		markDelivered,
		sendReaction,
		logout,
		sendRawWsMessage,
	};
});
