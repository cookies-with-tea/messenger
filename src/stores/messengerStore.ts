import { defineStore } from "pinia";
import { ref, computed, watch } from "vue";
import { useWebSocket } from "@vueuse/core";
import type { ChatResponseDTO, ChatMemberDTO, MessageResponseDTO, DeliveryStatus, WsServerEvent } from "@/types";
import { chatApi, messageApi, wsUrl } from "@/api";
import { useRouter } from "vue-router";

// UUID текущего пользователя — берётся из JWT payload
function parseCurrentUserId(): string {
	try {
		const token = localStorage.getItem("access_token") ?? "";
		if (!token) return "";
		const payload = JSON.parse(atob(token.split(".")[1]));
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
	const chatsLoading = ref(false);

	const messages = ref<Map<string, MessageResponseDTO[]>>(new Map());
	const messagesLoading = ref(false);

	const members = ref<Map<string, ChatMemberDTO[]>>(new Map());

	const activeChatId = ref<string | null>(null);

	// uuid → Set<user_uuid> кто сейчас печатает
	const typingMap = ref<Map<string, Set<string>>>(new Map());

	// ─── Computed: Динамический URL для WebSocket ─────────────────────
	// VueUse автоматически переподключится, когда изменится activeChatId
	const currentWsUrl = computed(() => {
		if (!activeChatId.value) return '';
		return wsUrl(activeChatId.value);
	});

	// ─── WebSocket Instance (VueUse) ──────────────────────────────────
	// immediate: false, чтобы не подключаться, пока нет активного чата
	// Мы контролируем подключение через watch или просто привязку к computed
	const { status, data, send, open, close } = useWebSocket(currentWsUrl, {
		immediate: false, // Подключаемся вручную при смене чата
		autoReconnect: {
			delay: 1000,
			retries: 3,
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

	// ─── WebSocket Message Handler ────────────────────────────────────
	// Обрабатываем входящие сообщения глобально для текущего сокета
	function handleWsMessage(event: string) {
		try {
			const raw = event;
			const ev: WsServerEvent = JSON.parse(raw);

			switch (ev.event) {
				case "new_message":
					_appendMessage(ev.payload);
					_bumpChat(ev.payload.chat_uuid, ev.payload.body, ev.payload.created_at);
					break;

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

				case "error":
					console.error("[WS Error]", ev.payload.message);
					break;

				case "pong":
					// Игнорируем пинг-понг
					break;

				default:
					console.warn("[WS] Unknown event:", ev.event);
			}
		} catch (e) {
			console.error("[WS] Failed to parse message:", e);
		}
	}

	// Подписываемся на данные только когда сокет активен
	watch(data, (newData) => {
		if (newData) {
			handleWsMessage(newData);
		}
	});

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

	// ─── Select & open chat ───────────────────────────────────────────
	async function selectChat(chatUuid: string) {
		// Если переходим в тот же чат, ничего не делаем
		if (activeChatId.value === chatUuid) return;

		// Смена активного чата автоматически триггерит изменение currentWsUrl
		// Но нам нужно явно закрыть старый сокет и открыть новый,
		// так как useWebSocket с computed URL может вести себя по-разному в зависимости от версии.
		// Надежный паттерн: close -> change ID -> open

		close();
		activeChatId.value = chatUuid;

		// Reset unread locally
		const chat = chats.value.find((c) => c.uuid === chatUuid);
		if (chat) chat.unread_count = 0;

		// Load messages if not cached
		if (!messages.value.has(chatUuid)) {
			await fetchMessages(chatUuid);
		}

		// Mark as read via API
		messageApi.markRead(chatUuid).catch(() => {});

		// Открываем новое соединение для нового URL
		// Небольшая задержка иногда нужна, чтобы computed успел обновиться,
		// но обычно open() сразу берет актуальное значение
		setTimeout(() => open(), 0);
	}

	// ─── Load messages ────────────────────────────────────────────────
	async function fetchMessages(chatUuid: string, beforeUuid?: string) {
		messagesLoading.value = true;
		try {
			const res = await messageApi.list(chatUuid, { limit: 50, before_uuid: beforeUuid });
			const incoming = res.data?.items ?? [];
			if (beforeUuid) {
				const existing = messages.value.get(chatUuid) ?? [];
				messages.value.set(chatUuid, [...incoming, ...existing]);
			} else {
				messages.value.set(chatUuid, incoming);
			}
		} finally {
			messagesLoading.value = false;
		}
	}

	// -- Load message by uuid ---
	async function fetchMessage(chatUuid: string) {
		const res = await messageApi.getByUuid(chatUuid);
		if (res.data) _appendMessage(res.data);
	}

	// ─── Send message via WS ──────────────────────────────────────────
	async function sendMessage(body: string, replyToUuid?: string) {
		const chatUuid = activeChatId.value;
		if (!chatUuid || !body.trim()) return;

		// Проверка статуса соединения
		if (status.value !== "OPEN") {
			// Попытка переподключения или фоллбэк на REST
			console.warn("WS not open, trying fallback or reconnect");
			// Можно попробовать open() здесь, если соединение разорвалось
		}

		const payload = {
			action: "send_message",
			payload: { body: body.trim(), reply_to_uuid: replyToUuid ?? null },
		};

		send(JSON.stringify(payload));

		// Оптимистичное обновление можно добавить здесь, если сервер не эхо-ответит мгновенно,
		// но в вашей архитектуре сервер присылает new_message, так что ждем события.
	}

	// ─── Edit message ─────────────────────────────────────────────────
	async function editMessage(msgUuid: string, body: string) {
		const chatUuid = activeChatId.value;
		if (!chatUuid) return;

		send(JSON.stringify({
			action: "edit_message",
			payload: { uuid: msgUuid, body },
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

	// ─── Typing events ────────────────────────────────────────────────
	function sendTyping(isTyping: boolean) {
		const chatUuid = activeChatId.value;
		if (!chatUuid) return;

		send(JSON.stringify({
			action: "typing",
			payload: { is_typing: isTyping },
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
		if (idx !== -1) list.splice(idx, 1, msg);
	}

	function _removeMessage(chatUuid: string, msgUuid: string) {
		const list = messages.value.get(chatUuid);
		if (!list) return;
		messages.value.set(
			chatUuid,
			list.filter((m) => m.uuid !== msgUuid),
		);
	}

	function _bumpChat(chatUuid: string, lastBody: string, lastAt: string) {
		const chat = chats.value.find((c) => c.uuid === chatUuid);
		if (!chat) return;
		chat.last_message_body = lastBody;
		chat.last_message_at = lastAt;
		const idx = chats.value.indexOf(chat);
		if (idx > 0) {
			chats.value.splice(idx, 1);
			chats.value.unshift(chat);
		}
	}

	function _applyStatus(chatUuid: string, _userUuid: string, statusVal: DeliveryStatus) {
		const list = messages.value.get(chatUuid);
		if (!list) return;
		list.forEach((m) => {
			if (m.my_status !== "read") m.my_status = statusVal;
		});
	}

	// ─── Initialise ───────────────────────────────────────────────────
	fetchChats();

	return {
		// state
		currentUserId,
		chats,
		chatsLoading,
		messages,
		messagesLoading,
		members,
		activeChatId,
		wsStatus: status, // Экспортируем статус сокета для UI (например, показать иконку подключения)
		// computed
		activeChat,
		activeMessages,
		typingUsersFor,
		memberName,
		// methods
		fetchChats,
		fetchMessages,
		selectChat,
		sendMessage,
		editMessage,
		deleteMessage,
		sendTyping,
	};
});
