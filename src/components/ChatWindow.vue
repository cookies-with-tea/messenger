<template>
	<div class="flex flex-col h-full bg-transparent">
		<ChatHeader :chat="messenger.activeChat!" />

		<!-- Messages -->
		<div
			ref="messagesEl"
			class="flex-1 overflow-y-auto scroll-smooth scrollbar-thin bg-transparent"
			@scroll="onScroll"
		>
      <div class="max-w-3xl mx-auto px-4 sm:px-6 py-6 sm:py-8 space-y-1 sm:space-y-1.5">
			  <!-- Load more -->
			  <div v-if="messenger.messagesLoading" class="flex justify-center py-2">
				  <span class="text-[10px] sm:text-xs font-mono text-muted animate-pulse font-bold tracking-widest uppercase">Scanning Stream...</span>
			  </div>

			  <template v-for="(group) in groupedMessages" :key="group.date">
				  <div class="flex items-center gap-3 py-4">
					  <div class="flex-1 h-px bg-white/5" />
					  <span class="text-[9px] sm:text-[10px] font-mono font-black text-muted px-4 py-1 rounded-full glass border border-white/5 uppercase tracking-widest">{{ group.date }}</span>
					  <div class="flex-1 h-px bg-white/5" />
				  </div>

				  <div v-for="(msg, mIdx) in group.messages" :key="msg.uuid" :id="`msg-${msg.uuid}`" class="mb-0.5">
					  <MessageBubble
						  :message="msg"
						  :is-group="messenger.activeChat?.chat_type === 'group'"
						  :show-avatar="shouldShowAvatar(group.messages, mIdx)"
              @image-click="openZoom"
					  />
				  </div>
			  </template>

			  <!-- Typing -->
			  <div v-if="typingUsers.length > 0" class="pt-2">
				  <TypingIndicator :user-ids="typingUsers" />
			  </div>

			  <div ref="bottomAnchor" />
      </div>
		</div>


		<MessageInput @send="handleSend" @typing="messenger.sendTyping(true)" @stop-typing="messenger.sendTyping(false)" />

    <ImageZoomModal 
      :src="messenger.imageZoomSrc" 
      :is-open="messenger.isImageZoomOpen" 
      @close="messenger.closeZoom" 
    />
	</div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from "vue";
import { useMessengerStore } from "@/stores/messengerStore";
import type { MessageResponseDTO } from "@/types";
import ChatHeader from "./ChatHeader.vue";
import MessageBubble from "./MessageBubble.vue";
import MessageInput from "./MessageInput.vue";
import TypingIndicator from "./TypingIndicator.vue";
import ImageZoomModal from "./ImageZoomModal.vue";

const messenger = useMessengerStore();
const bottomAnchor = ref<HTMLDivElement>();
const messagesEl = ref<HTMLDivElement>();

const typingUsers = computed(() => (messenger.activeChatId ? messenger.typingUsersFor(messenger.activeChatId) : []));

function shouldShowAvatar(messages: MessageResponseDTO[], idx: number): boolean {
	if (idx === 0) return true;
	return messages[idx - 1].sender_uuid !== messages[idx].sender_uuid;
}

const groupedMessages = computed(() => {
	const groups: { date: string; messages: MessageResponseDTO[] }[] = [];
	let current: { date: string; messages: MessageResponseDTO[] } | null = null;
	for (const msg of messenger.activeMessages) {
		const label = formatDate(new Date(msg.created_at));
		if (!current || current.date !== label) {
			current = { date: label, messages: [] };
			groups.push(current);
		}
		current.messages.push(msg);
	}
	return groups;
});

function formatDate(d: Date): string {
	const now = new Date();
	const diff = now.getTime() - d.getTime();
	if (diff < 86_400_000 && d.getDate() === now.getDate()) return "Today";
	if (diff < 172_800_000) return "Yesterday";
	return d.toLocaleDateString("en", { month: "short", day: "numeric" });
}

function scrollToBottom(instant = false) {
	nextTick(() => {
		if (bottomAnchor.value) {
			bottomAnchor.value.scrollIntoView({ behavior: instant ? "instant" : "smooth" });
		}
	});
}

const allLoaded = ref(false);
let lastFetchedOldestUuid: string | null = null;

// Load older messages on scroll to top
async function onScroll() {
	if (!messagesEl.value || messenger.messagesLoading || allLoaded.value) return;
	
	if (messagesEl.value.scrollHeight <= messagesEl.value.clientHeight) return;

	if (messagesEl.value.scrollTop < 60 && messenger.activeMessages.length > 0) {
		const oldestUuid = messenger.activeMessages[0]?.uuid;
		if (oldestUuid && messenger.activeChatId && oldestUuid !== lastFetchedOldestUuid) {
			lastFetchedOldestUuid = oldestUuid;
			const oldLength = messenger.activeMessages.length;
			await messenger.fetchMessages(messenger.activeChatId, oldestUuid);
			if (messenger.activeMessages.length === oldLength) {
				allLoaded.value = true;
			}
		}
	}
}

watch(() => messenger.activeChatId, () => {
	lastFetchedOldestUuid = null;
	allLoaded.value = false;
});

watch(
	() => messenger.activeChatId,
	(v) => {
		if (v) scrollToBottom(true);
	},
	{ immediate: true },
);

watch(
	() => messenger.activeMessages.length,
	() => scrollToBottom(),
);

watch(
	() => typingUsers.value.length,
	() => scrollToBottom(),
);

function handleSend(text: string) {
	messenger.sendMessage(text);
}
</script>
