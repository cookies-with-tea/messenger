import { createRouter, createWebHistory } from "vue-router";
import { tokenStore } from "@/api";

const router = createRouter({
	history: createWebHistory(),
	routes: [
		{
			path: "/login",
			name: "login",
			component: () => import("@/views/LoginView.vue"),
			meta: { public: true },
		},
		{
			path: "/register",
			name: "register",
			component: () => import("@/views/RegisterView.vue"),
			meta: { public: true },
		},
		// Backend sends confirm link to: /confirm-register?key=UUID
		{
			path: "/confirm-register",
			name: "confirm-register",
			component: () => import("@/views/ConfirmRegisterView.vue"),
			meta: { public: true },
		},
		{
			path: "/",
			name: "messenger",
			component: () => import("@/views/MessengerView.vue"),
			meta: { requiresAuth: true },
			children: [
				{
					path: "",
					name: "messenger-empty",
					component: () => import("@/components/EmptyState.vue"),
				},
				{
					path: "chat/:chatId",
					name: "chat",
					component: () => import("@/components/ChatWindow.vue"),
					props: true,
				},
			],
		},
		// Fallback
		{ path: "/:pathMatch(.*)*", redirect: "/" },
	],
});

// ─── Auth guard ───────────────────────────────────────────────────
router.beforeEach(async (to) => {
	const loggedIn = tokenStore.isLoggedIn();

	// Try token refresh if access expired but refresh still valid
	if (!loggedIn && tokenStore.refreshToken()) {
		try {
			const { authApi } = await import("@/api");
			const res = await authApi.refresh(tokenStore.refreshToken()!);
			if (res.data) {
				tokenStore.setTokens(res.data);
				// re-check
				if (to.meta.requiresAuth) return true;
			} else {
				tokenStore.clear();
			}
		} catch {
			console.log("eer");
			tokenStore.clear();
		}
	}

	if (to.meta.requiresAuth && !tokenStore.isLoggedIn()) {
		return { name: "login" };
	}

	if (to.meta.public && tokenStore.isLoggedIn()) {
		return { name: "messenger" };
	}

	return true;
});

export default router;
