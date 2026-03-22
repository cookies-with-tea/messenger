import type {
	ApiResponse,
	ApiResponseWithPagination,
	ChatResponseDTO,
	ChatMemberDTO,
	MessageResponseDTO,
	UserResponseDTO,
	ChatMediaCountsDTO,
	MessageReceiptDTO,
} from "@/types";

import { useToastStore } from "@/stores/toastStore";
import { ref } from "vue";

const IS_DEV = import.meta.env.DEV;
const BASE = IS_DEV ? "" : (import.meta.env.VITE_BASE_URL ?? "http://localhost:8080");

// Reactive token for WebSockets
const accessToken = ref(localStorage.getItem("access_token"));

function authHeaders(): Record<string, string> {
	const token = localStorage.getItem("access_token");
	return token ? { Authorization: `Bearer ${token}` } : {};
}

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
	const toast = useToastStore();
	try {
		const res = await fetch(`${BASE}${path}`, {
			...options,
			headers: { 
				"Content-Type": "application/json", 
				...authHeaders(),
				...options.headers 
			},
		});

		const data = await res.json();

		if (!res.ok) {
			const errorMsg = data.errors?.[0] || data.messages?.[0] || `Request failed with status ${res.status}`;
			toast.error(errorMsg);
			throw new Error(errorMsg);
		}

		return data;
	} catch (err: any) {
		if (!(err instanceof Error)) {
			toast.error("Network error. Please check your connection.");
		}
		throw err;
	}
}

async function get<T>(path: string): Promise<T> {
	return request<T>(path);
}

async function post<T>(path: string, body?: unknown): Promise<T> {
	return request<T>(path, {
		method: "POST",
		body: body != null ? JSON.stringify(body) : undefined,
	});
}

async function put<T>(path: string, body?: unknown): Promise<T> {
	return request<T>(path, {
		method: "PUT",
		body: body != null ? JSON.stringify(body) : undefined,
	});
}

async function patch<T>(path: string, body?: unknown): Promise<T> {
	return request<T>(path, {
		method: "PATCH",
		body: body != null ? JSON.stringify(body) : undefined,
	});
}

async function del<T>(path: string): Promise<T> {
	return request<T>(path, {
		method: "DELETE",
	});
}

// ─── Chats ────────────────────────────────────────────────────────
export const chatApi = {
	list: (page = 1, limit = 30) =>
		get<ApiResponseWithPagination<ChatResponseDTO>>(`/api/v1/chats?page=${page}&limit=${limit}`),

	get: (uuid: string) => get<ApiResponse<ChatResponseDTO>>(`/api/v1/chats/${uuid}`),

	create: (body: { name?: string; description?: string; chat_type: string; member_uuids: string[] }) =>
		post<ApiResponse<string>>("/api/v1/chats", body),

	update: (uuid: string, body: { name?: string; description?: string; avatar?: string; is_archived?: boolean }) =>
		put<ApiResponse<ChatResponseDTO>>(`/api/v1/chats/${uuid}`, body),

	setAlias: (uuid: string, alias: string | null) =>
		patch<ApiResponse<null>>(`/api/v1/chats/${uuid}/alias`, { alias }),

	leave: (uuid: string) => del<ApiResponse<null>>(`/api/v1/chats/${uuid}`),
	getMediaCounts: (uuid: string) => get<ApiResponse<ChatMediaCountsDTO>>(`/api/v1/chats/${uuid}/media/counts`),
	getMedia: (uuid: string, params: { media_type?: string; page?: number; limit?: number } = {}) => {
		const q = new URLSearchParams();
		if (params.media_type) q.set("media_type", params.media_type);
		if (params.page) q.set("page", String(params.page));
		if (params.limit) q.set("limit", String(params.limit));
		return get<ApiResponse<MessageResponseDTO[]>>(`/api/v1/chats/${uuid}/media?${q}`);
	},
};

// ─── Members ──────────────────────────────────────────────────────
export const memberApi = {
	list: (chatUuid: string) => get<ApiResponse<ChatMemberDTO[]>>(`/api/v1/chats/${chatUuid}/members`),
};

// ─── Messages ─────────────────────────────────────────────────────
export const messageApi = {
	list: (chatUuid: string, params: { page?: number; limit?: number; before_uuid?: string } = {}) => {
		const q = new URLSearchParams();
		if (params.page) q.set("page", String(params.page));
		if (params.limit) q.set("limit", String(params.limit));
		if (params.before_uuid) q.set("before_uuid", params.before_uuid);
		return get<ApiResponseWithPagination<MessageResponseDTO>>(`/api/v1/chats/${chatUuid}/messages?${q}`);
	},

	send: (chatUuid: string, body: string, replyToUuid?: string) =>
		post<ApiResponse<MessageResponseDTO>>(`/api/v1/chats/${chatUuid}/messages`, {
			body,
			reply_to_uuid: replyToUuid ?? null,
		}),

	edit: (chatUuid: string, msgUuid: string, body: string) =>
		put<ApiResponse<MessageResponseDTO>>(`/api/v1/chats/${chatUuid}/messages/${msgUuid}`, { body }),

	delete: (chatUuid: string, msgUuid: string) =>
		del<ApiResponse<null>>(`/api/v1/chats/${chatUuid}/messages/${msgUuid}`),

	markDelivered: (chatUuid: string) => post<ApiResponse<null>>(`/api/v1/chats/${chatUuid}/messages/delivered`),

	markRead: (chatUuid: string) => post<ApiResponse<null>>(`/api/v1/chats/${chatUuid}/messages/read`),

	getByUuid: (chatUuid: string) => get<ApiResponse<MessageResponseDTO>>(`/api/v1/chats/${chatUuid}`),

	getMessageReceipts: (chatUuid: string, msgUuid: string) =>
		get<ApiResponse<MessageReceiptDTO[]>>(`/api/v1/chats/${chatUuid}/messages/${msgUuid}/receipts`),

	search: (chatUuid: string, q: string, limit = 50) =>
		get<ApiResponse<MessageResponseDTO[]>>(`/api/v1/chats/${chatUuid}/search?q=${encodeURIComponent(q)}&limit=${limit}`),

	globalSearch: (q: string, limit = 50) =>
		get<ApiResponse<MessageResponseDTO[]>>(`/api/v1/chats/search?q=${encodeURIComponent(q)}&limit=${limit}`),
};

// ─── WebSocket URL builder ─────────────────────────────────────────
const WS_BASE = import.meta.env.VITE_BASE_WS_URL ?? "ws://localhost:8000";

/** Per-chat WebSocket (старый, для совместимости) */
export function wsUrl(chatUuid: string): string {
  const token = accessToken.value ?? "";
  return `${WS_BASE}/ws/chats/${chatUuid}?token=${token}`
}

/** Глобальный WebSocket — один канал на пользователя, все чаты */
export function wsUserUrl(): string {
  const token = accessToken.value ?? "";
  return `${WS_BASE}/ws/user?token=${token}`
}

// ─── Users ────────────────────────────────────────────────────────
export const userApi = {
	search: (query: string, limit = 20) => {
		const q = new URLSearchParams({ limit: String(limit) });
		if (query) q.set("search", query);
		return get<ApiResponseWithPagination<UserResponseDTO>>(`/api/v1/user?${q}`);
	},
	get: (uuid: string) => get<ApiResponse<UserResponseDTO>>(`/api/v1/user/${uuid}`),
	update: (uuid: string, body: Partial<UserResponseDTO>) => patch<ApiResponse<UserResponseDTO>>(`/api/v1/user/${uuid}`, body),
};

// ─── Media ────────────────────────────────────────────────────────
export const mediaApi = {
	upload: (file: File | Blob, title?: string, alt?: string) => {
		const fd = new FormData();
		fd.append("file", file);
		if (title) fd.append("title", title);
		if (alt) fd.append("alt", alt);

		return fetch(`${BASE}/api/v1/media`, {
			method: "POST",
			headers: { ...authHeaders() },
			body: fd,
		}).then((res) => res.json() as Promise<ApiResponse<{ uuid: string; url: string }>>);
	},
};

// ─── Auth ─────────────────────────────────────────────────────────
export interface AuthResponseDTO {
	access_token: string;
	access_expires_in: number;
	refresh_token: string;
	refresh_expires_in: number;
}

export const authApi = {
	// Step 1: send email → backend emails a confirm link
	register: (email: string) => post<ApiResponse<null>>("/api/v1/auth/register", { email }),

	// Step 2: confirm link → /confirm-register?key=UUID
	checkKey: (key: string) => post<ApiResponse<null>>("/api/v1/auth/register/key", { key }),

	// Step 3: create user with password
	createUser: (body: { email: string; password: string; first_name?: string; last_name?: string; phone?: string }) =>
		post<ApiResponse<null>>("/api/v1/user", body),

	// Login
	login: (email: string, password: string) =>
		post<ApiResponse<AuthResponseDTO>>("/api/v1/auth/login", { email, password }),

	// Refresh
	refresh: (refresh_token: string) => post<ApiResponse<AuthResponseDTO>>("/api/v1/auth/refresh", { refresh_token }),

	// Logout (just clear tokens locally — backend may not have explicit endpoint)
	logout: () => post<ApiResponse<null>>("/api/v1/auth/logout"),
};

// ─── Token helpers ────────────────────────────────────────────────
export const tokenStore = {
	setTokens(data: AuthResponseDTO) {
		localStorage.setItem("access_token", data.access_token);
		localStorage.setItem("refresh_token", data.refresh_token);
		localStorage.setItem("access_expires_in", String(Date.now() + data.access_expires_in * 1000));
		localStorage.setItem("refresh_expires_in", String(Date.now() + data.refresh_expires_in * 1000));
		accessToken.value = data.access_token;
	},
	clear() {
		localStorage.removeItem("access_token");
		localStorage.removeItem("refresh_token");
		localStorage.removeItem("access_expires_in");
		localStorage.removeItem("refresh_expires_in");
		accessToken.value = null;
	},
	isLoggedIn(): boolean {
		const exp = localStorage.getItem("access_expires_in");
		return !!exp && Date.now() < Number(exp);
	},
	accessToken(): string | null {
		return localStorage.getItem("access_token");
	},
	refreshToken(): string | null {
		return localStorage.getItem("refresh_token");
	},
};
