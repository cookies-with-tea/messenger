import { defineStore } from "pinia";
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { useWebSocket } from "@vueuse/core";
import type { ChatResponseDTO, ChatMemberDTO, MessageResponseDTO, UserResponseDTO, DeliveryStatus, ChatMediaCountsDTO } from "@/types";
import { chatApi, messageApi, mediaApi, userApi, wsUserUrl, tokenStore } from "@/api";
import { showNotification } from "@/api/notifications";
import { useSettingsStore } from "@/stores/settingsStore";
import { useRouter } from "vue-router";

// UUID текущего пользователя — берётся из JWT payload
function parseCurrentUserId(): string {
	try {
		const token = localStorage.getItem("access_token") ?? "";
		if (!token) return "";
		const parts = token.split(".");
		if (parts.length < 2) return "";
		const payload = JSON.parse(atob(parts[1] || ""));
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
	const currentUserProfileLoading = ref(false);
	const chatsLoading = ref(false);

	const messages = ref<Map<string, MessageResponseDTO[]>>(new Map());
	const messagesLoading = ref(false);

	const members = ref<Map<string, ChatMemberDTO[]>>(new Map());

	const activeChatId = ref<string | null>(null);

	// uuid → Set<user_uuid> кто сейчас печатает
	const typingMap = ref<Map<string, Set<string>>>(new Map());

	// Organization State
	const activeFolder = ref<'all' | 'unread' | 'groups' | 'archived'>('all')
	
	const _storedPinned = JSON.parse(localStorage.getItem('pinned_chats') || '[]')
	const _storedArchived = JSON.parse(localStorage.getItem('archived_chats') || '[]')

	const pinnedChatUuids = ref<Set<string>>(new Set(_storedPinned))
	const archivedChatUuids = ref<Set<string>>(new Set(_storedArchived))
	const now = ref(new Date());

	// Update 'now' every 60s for last-seen reactivity
	let nowTimer: any = null;
	function startNowTimer() {
		if (nowTimer) clearInterval(nowTimer);
		nowTimer = setInterval(() => {
			now.value = new Date();
		}, 60000);
	}

	// Edition & Reply State
	const replyingToMessage = ref<MessageResponseDTO | null>(null);
	const editingMessage = ref<MessageResponseDTO | null>(null);

	// Track which chats have had their full history fetched
	const loadedChats = ref<Set<string>>(new Set());

	// Profile Modal State
	const selectedProfileUserId = ref<string | null>(null);
	const selectedProfile = ref<UserResponseDTO | null>(null);
	const isProfileModalOpen = ref(false);
	const profileLoading = ref(false);
	const mediaCounts = ref<Map<string, ChatMediaCountsDTO>>(new Map());
	const sharedMedia = ref<Map<string, MessageResponseDTO[]>>(new Map());
	const sharedMediaLoading = ref(false);
	
	// Search Modal State
	const isSearchModalOpen = ref(false);
	const isGlobalSearchModalOpen = ref(false);
	
	// Image Zoom State
	const isImageZoomOpen = ref(false);
	const imageZoomSrc = ref("");

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
	const pinnedMessages = computed(() => activeMessages.value.filter(m => m.is_pinned));

	const filteredChats = computed(() => {
		let list = [...chats.value]

		// Filter by active folder
		if (activeFolder.value === 'archived') {
			list = list.filter(c => archivedChatUuids.value.has(c.uuid))
		} else {
			// In non-archived folders, hide archived chats
			list = list.filter(c => !archivedChatUuids.value.has(c.uuid))

			if (activeFolder.value === 'unread') {
				list = list.filter(c => c.unread_count && (c.unread_count as number) > 0)
			} else if (activeFolder.value === 'groups') {
				list = list.filter(c => c.chat_type === 'group')
			}
		}

		// Sort: Pinned first, then by last message time
		return list.sort((a, b) => {
			const aPinned = pinnedChatUuids.value.has(a.uuid)
			const bPinned = pinnedChatUuids.value.has(b.uuid)

			if (aPinned && !bPinned) return -1
			if (!aPinned && bPinned) return 1

			// Fallback to time sorting
			const aTime = a.last_message_at ? new Date(a.last_message_at).getTime() : 0
			const bTime = b.last_message_at ? new Date(b.last_message_at).getTime() : 0
			return bTime - aTime
		})
	})

	function typingUsersFor(chatUuid: string): string[] {
		return Array.from(typingMap.value.get(chatUuid) ?? []);
	}

	const isAnyModalOpen = computed(() => {
		return isProfileModalOpen.value || 
			   isSearchModalOpen.value || 
			   isGlobalSearchModalOpen.value ||
			   isImageZoomOpen.value;
	});

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
			const evData = JSON.parse(event) as any;
			console.debug("[WS:ParsedEvent]", evData.event, evData);

			const eventType = evData.event;
			const payload = evData.payload;

			// Handle both prefixed and non-prefixed (global) events
			switch (eventType) {
				case "new_message":
				case "NewMessage": {
					const msg = payload;
					_appendMessage(msg);
					_bumpChat(msg.chat_uuid, msg.body, msg.created_at);
					
					if (msg.sender_uuid !== currentUserId.value) {
						if (msg.chat_uuid === activeChatId.value) {
							markRead(msg.chat_uuid);
						} else {
							markDelivered(msg.chat_uuid);
						}

						const settings = useSettingsStore();
						if (settings.notificationsEnabled) {
							const chat = chats.value.find(c => c.uuid === msg.chat_uuid);
							showNotification(chat?.name || msg.sender?.first_name || "New Message", {
								body: msg.body,
								icon: msg.sender?.avatar?.url || "/vite.svg"
							});
						}
					}
					break;
				}

				case "message_edited":
				case "MessageEdited":
					_replaceMessage(payload);
					break;

				case "message_deleted":
				case "MessageDeleted":
					_removeMessage(payload.chat_uuid, payload.uuid);
					break;

				case "status_updated":
				case "StatusUpdated":
					_applyStatus(payload.chat_uuid, payload.user_uuid, payload.status);
					break;

				case "message_pinned":
				case "MessagePinned": {
					const { chat_uuid, uuid, is_pinned } = payload;
					_updatePinnedStatus(chat_uuid, uuid, is_pinned);
					break;
				}

				case "typing":
				case "Typing": {
					const { chat_uuid, user_uuid, is_typing } = payload;
					if (user_uuid === currentUserId.value) return;
					const set = typingMap.value.get(chat_uuid) ?? new Set();
					is_typing ? set.add(user_uuid) : set.delete(user_uuid);
					typingMap.value.set(chat_uuid, new Set(set));
					break;
				}

				case "user_status_changed":
				case "UserStatusChanged": {
					const { user_uuid, is_online, last_seen_at } = payload;
					_updateUserStatus(user_uuid, is_online, last_seen_at);
					break;
				}

				case "message_reaction_updated":
				case "MessageReactionUpdated": {
					const { chat_uuid, message_uuid, user_uuid, emoji, is_added } = payload;
					_updateReaction(chat_uuid, message_uuid, user_uuid, emoji, is_added);
					break;
				}

				// ─── WebRTC Signaling ───
				case "call_offer": {
					import("./callStore").then(({ useCallStore }) => {
						// @ts-ignore
						useCallStore().receiveOffer(payload.chat_uuid, payload.caller_uuid, payload.sdp);
					});
					break;
				}
				case "call_answer": {
					import("./callStore").then(({ useCallStore }) => {
						// @ts-ignore
						useCallStore().receiveAnswer(payload.chat_uuid, payload.responder_uuid, payload.sdp);
					});
					break;
				}
				case "ice_candidate": {
					import("./callStore").then(({ useCallStore }) => {
						// @ts-ignore
						useCallStore().receiveIceCandidate(payload.chat_uuid, payload.candidate, payload.sdp_mid, payload.sdp_m_line_index);
					});
					break;
				}
				case "call_reject": {
					import("./callStore").then(({ useCallStore }) => {
						// @ts-ignore
						useCallStore().handleRemoteCallReject(payload.chat_uuid);
					});
					break;
				}
				case "call_end": {
					import("./callStore").then(({ useCallStore }) => {
						// @ts-ignore
						useCallStore().handleRemoteCallEnd(payload.chat_uuid);
					});
					break;
				}

				case "error":
					console.error("[WS Error]", evData.payload.message);
					break;

				case "pong":
					// Игнорируем пинг-понг
					break;

				default:
					console.warn("[WS] Unknown event:", (evData as any).event);
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

	// Reload profile if user ID changes (e.g. after login/switching accounts)
	watch(currentUserId, (newId) => {
		if (newId) {
			fetchCurrentUser();
		} else {
			currentUserProfile.value = null;
		}
	});

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
		if (!currentUserId.value) {
			console.warn("[MessengerStore] Cannot fetch current user: currentUserId is empty");
			return;
		}
		currentUserProfileLoading.value = true;
		try {
			console.debug("[MessengerStore] Fetching current user profile for:", currentUserId.value);
			const res = await userApi.get(currentUserId.value);
			currentUserProfile.value = res.data;
		} catch (e) {
			console.error("[Store] Failed to fetch current user profile:", e);
		} finally {
			currentUserProfileLoading.value = false;
		}
	}

	async function fetchUserProfile(userId: string) {
		profileLoading.value = true;
		try {
			const res = await userApi.get(userId);
			selectedProfile.value = res.data;
		} catch (e) {
			console.error("[Store] Failed to fetch user profile:", e);
		} finally {
			profileLoading.value = false;
		}
	}
	
	async function fetchMediaCounts(chatUuid: string) {
		try {
			const res = await chatApi.getMediaCounts(chatUuid);
			if (res.data) {
				mediaCounts.value.set(chatUuid, res.data);
			}
		} catch (e) {
			console.error("[Store] Failed to fetch media counts:", e);
		}
	}

	async function fetchSharedMedia(chatUuid: string, type?: string) {
		sharedMediaLoading.value = true;
		try {
			const res = await chatApi.getMedia(chatUuid, { media_type: type });
			if (res.data) {
				sharedMedia.value.set(chatUuid + (type || ''), res.data);
			}
		} catch (e) {
			console.error("[Store] Failed to fetch shared media:", e);
		} finally {
			sharedMediaLoading.value = false;
		}
	}

	function openProfile(userId: string) {
		selectedProfileUserId.value = userId;
		selectedProfile.value = null;
		isProfileModalOpen.value = true;
		fetchUserProfile(userId);
	}

	function closeProfile() {
		isProfileModalOpen.value = false;
		setTimeout(() => {
			selectedProfileUserId.value = null;
			selectedProfile.value = null;
		}, 300);
	}

	// ─── Select & open chat ───────────────────────────────────────────
	// WS НЕ переподключается — он глобальный и уже открыт
	async function selectChat(chatUuid: string) {
		if (!chatUuid || chatUuid === 'undefined') return;
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
		if (!chatUuid || chatUuid === 'undefined') return;
		if (messagesLoading.value) return;
		messagesLoading.value = true;
		try {
			const res = await messageApi.list(chatUuid, { limit: 50, before_uuid: beforeUuid });
			// Backend returns DESC (newest first), we want ASC (oldest first) in store for easy appending
			const incoming = (res.data?.items ?? []).reverse();
			
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

	function togglePinMessage(msgUuid: string, isPinned: boolean) {
		const chatUuid = activeChatId.value;
		if (!chatUuid) return;

		send(JSON.stringify({
			action: "toggle_pin_message",
			payload: { chat_uuid: chatUuid, uuid: msgUuid, is_pinned: isPinned },
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

		// 4. Обновляем selectedProfile (если открыт)
		if (selectedProfile.value && selectedProfile.value.uuid === userUuid) {
			selectedProfile.value.is_online = isOnline;
			selectedProfile.value.last_seen_at = lastSeenAt;
		}
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

	function _updatePinnedStatus(chatUuid: string, msgUuid: string, isPinned: boolean) {
		const list = messages.value.get(chatUuid);
		if (!list) return;

		const msg = list.find((m) => m.uuid === msgUuid);
		if (msg) {
			msg.is_pinned = isPinned;
			messages.value.set(chatUuid, [...list]);
		}
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
			
			// Move to top of the underlying array to trigger filteredChats naturally
			const idx = chats.value.findIndex(c => c.uuid === chatUuid);
			if (idx !== -1) {
				const moved = chats.value.splice(idx, 1)[0];
				if (moved) {
					chats.value.unshift(moved);
				}
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
		currentUserProfile.value = null;
		chats.value = [];
		messages.value.clear();
		members.value.clear();
		activeChatId.value = null;
		typingMap.value.clear();
		loadedChats.value.clear();
		activeFolder.value = 'all'
		pinnedChatUuids.value = new Set()
		archivedChatUuids.value = new Set()

		router.push("/login");
	}

	// ─── Organization Actions ─────────────────────────────────────────
	function togglePinChat(chatUuid: string) {
		const next = new Set(pinnedChatUuids.value)
		if (next.has(chatUuid)) {
			next.delete(chatUuid)
		} else {
			next.add(chatUuid)
		}
		pinnedChatUuids.value = next
		localStorage.setItem('pinned_chats', JSON.stringify(Array.from(next)))
	}

	function toggleArchiveChat(chatUuid: string) {
		const next = new Set(archivedChatUuids.value)
		if (activeFolder.value === 'archived' || next.has(chatUuid)) {
			next.delete(chatUuid)
		} else {
			next.add(chatUuid)
			// If we archive an active chat, deselect it
			if (activeChatId.value === chatUuid) activeChatId.value = null
		}
		archivedChatUuids.value = next
		localStorage.setItem('archived_chats', JSON.stringify(Array.from(next)))
	}

	function setFolder(folder: 'all' | 'unread' | 'groups' | 'archived') {
		activeFolder.value = folder
		// If current active chat is not in the new folder, we keep it selected but user might want to switch
	}

	// ─── Initialization ───────────────────────────────────────────────
	onMounted(() => {
		startNowTimer();
	});

	// Sync activeChatId with route
	watch(() => router.currentRoute.value.params.chatId, (newId) => {
		if (newId && typeof newId === 'string' && newId !== 'undefined') {
			selectChat(newId);
		} else if (!newId) {
			activeChatId.value = null;
		}
	}, { immediate: true });

	onUnmounted(() => {
		if (nowTimer) clearInterval(nowTimer);
	});

	return {
		// state
		currentUserId,
		currentUserProfile,
		currentUserProfileLoading,
		chats,
		chatsLoading,
		messages,
		messagesLoading,
		members,
		activeChatId,
		wsStatus: status,
		replyingToMessage,
		editingMessage,
		now, // Add 'now' to the returned state
		// computed
		activeChat,
		activeMessages,
		pinnedMessages,
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
		togglePinMessage,
		logout,
		sendRawWsMessage,
		// profile
		selectedProfileUserId,
		selectedProfile,
		isProfileModalOpen,
		profileLoading,
		openProfile,
		closeProfile,
		mediaCounts,
		sharedMedia,
		sharedMediaLoading,
		fetchMediaCounts,
		fetchSharedMedia,
		// modals
		isSearchModalOpen,
		isGlobalSearchModalOpen,
		isImageZoomOpen,
		isAnyModalOpen,
		imageZoomSrc,
		openZoom(src: string) {
			imageZoomSrc.value = src;
			isImageZoomOpen.value = true;
		},
		closeZoom() {
			isImageZoomOpen.value = false;
		},
		// updates
		async updateProfile(payload: any) {
			if (!currentUserId.value) return;
			try {
				const res = await userApi.update(currentUserId.value, payload);
				if (res.data) {
					currentUserProfile.value = res.data;
					// Update in selectedProfile if viewing own
					if (selectedProfileUserId.value === currentUserId.value) {
						selectedProfile.value = res.data;
					}
				}
				return res;
			} catch (e) {
				console.error("[Store] Failed to update profile:", e);
				throw e;
			}
		},
		async updateContactAlias(chatUuid: string, alias: string | null) {
			try {
				await chatApi.setAlias(chatUuid, alias);
				// Update local state
				const chat = chats.value.find(c => c.uuid === chatUuid);
				if (chat) {
					chat.alias = alias;
				}
			} catch (e) {
				console.error("[Store] Failed to update contact alias:", e);
				throw e;
			}
		},
		// organization
		activeFolder,
		pinnedChatUuids,
		archivedChatUuids,
		filteredChats,
		togglePinChat,
		toggleArchiveChat,
		setFolder
	};
});
