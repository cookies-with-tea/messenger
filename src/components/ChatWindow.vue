<template>
	<div class="flex flex-col h-full bg-transparent overflow-hidden">
		<ChatHeader :chat="messenger.activeChat!" />

		<!-- Messages -->
		<DynamicScroller
			:items="flattenedItems"
			:min-item-size="40"
			class="flex-1 scrollbar-thin bg-transparent"
			@scroll="onScroll"
			ref="scroller"
      key-field="id"
		>
      <template v-slot="{ item, index, active }">
        <DynamicScrollerItem
          :item="item"
          :active="active"
          :data-index="index"
          :size-dependencies="[item.message?.body, item.message?.media]"
        >
          <div v-if="item.type === 'date'" class="max-w-3xl mx-auto px-4 sm:px-6">
            <div class="flex items-center gap-3 py-4">
              <div class="flex-1 h-px bg-white/5" />
              <span class="text-[9px] sm:text-[10px] font-mono font-black text-muted px-4 py-1 rounded-full glass border border-white/5 uppercase tracking-widest">{{ item.date }}</span>
              <div class="flex-1 h-px bg-white/5" />
            </div>
          </div>
          <div v-else-if="item.type === 'message'" :id="`msg-${item.message.uuid}`" class="mb-0.5 max-w-3xl mx-auto px-4 sm:px-6">
            <MessageBubble
              :message="item.message"
              :is-group="messenger.activeChat?.chat_type === 'group'"
              :show-avatar="item.showAvatar"
              @image-click="messenger.openZoom"
            />
          </div>
          <div v-else-if="item.type === 'typing'" class="max-w-3xl mx-auto px-4 sm:px-6 pt-2 pb-8">
            <TypingIndicator :user-ids="typingUsers" />
          </div>
        </DynamicScrollerItem>
      </template>
      <template #before>
        <div v-if="messenger.messagesLoading" class="flex justify-center py-4">
          <span class="text-[10px] sm:text-xs font-mono text-muted animate-pulse font-bold tracking-widest uppercase">Scanning Stream...</span>
        </div>
      </template>
		</DynamicScroller>

		<MessageInput 
      ref="messageInput"
      @send="handleSend" 
      @typing="messenger.sendTyping(true)" 
      @stop-typing="messenger.sendTyping(false)" 
    />

    <ImageZoomModal 
      :src="messenger.imageZoomSrc" 
      :is-open="messenger.isImageZoomOpen" 
      @close="messenger.closeZoom" 
    />
	</div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted } from "vue";
import { useMessengerStore } from "@/stores/messengerStore";
import type { MessageResponseDTO } from "@/types";
import ChatHeader from "./ChatHeader.vue";
import MessageBubble from "./MessageBubble.vue";
import MessageInput from "./MessageInput.vue";
import TypingIndicator from "./TypingIndicator.vue";
import ImageZoomModal from "./ImageZoomModal.vue";

const messenger = useMessengerStore();
const scroller = ref<any>();
const messageInput = ref<any>();
const allLoaded = ref(false);
let lastFetchedOldestUuid: string | null = null;

const typingUsers = computed(() => (messenger.activeChatId ? messenger.typingUsersFor(messenger.activeChatId) : []));

function shouldShowAvatar(messages: MessageResponseDTO[], idx: number): boolean {
	if (idx === 0) return true;
  const prev = messages[idx - 1];
  const curr = messages[idx];
  if (!prev || !curr) return true;
	return prev.sender_uuid !== curr.sender_uuid;
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

const flattenedItems = computed(() => {
  const items: any[] = [];
  for (const group of groupedMessages.value) {
    items.push({
      id: `date-${group.date}`,
      type: 'date',
      date: group.date
    });
    group.messages.forEach((msg, mIdx) => {
      items.push({
        id: msg.uuid,
        type: 'message',
        message: msg,
        showAvatar: shouldShowAvatar(group.messages, mIdx)
      });
    });
  }
  if (typingUsers.value.length > 0) {
    items.push({
      id: 'typing-indicator',
      type: 'typing'
    });
  }
  return items;
});

function formatDate(d: Date): string {
	const now = new Date();
	const diff = now.getTime() - d.getTime();
	if (diff < 86_400_000 && d.getDate() === now.getDate()) return "Today";
	if (diff < 172_800_000) return "Yesterday";
	return d.toLocaleDateString("en", { month: "short", day: "numeric" });
}

function scrollToBottom() {
	nextTick(() => {
		if (scroller.value) {
			scroller.value.scrollToBottom();
      // Multi-stage fallback for virtual scroller layout
      setTimeout(() => scroller.value?.scrollToBottom(), 50);
      setTimeout(() => scroller.value?.scrollToBottom(), 200);
		}
	});
}

// Load older messages on scroll to top
async function onScroll() {
  const el = scroller.value?.$el;
	if (!el || messenger.messagesLoading || allLoaded.value) return;
	
	if (el.scrollHeight <= el.clientHeight) return;

	if (el.scrollTop < 60 && messenger.activeMessages.length > 0) {
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

// Lifecycle and Watchers
onMounted(() => {
  scrollToBottom();
  messageInput.value?.focus();
});

// Watch for chat changes
watch(() => messenger.activeChatId, () => {
	lastFetchedOldestUuid = null;
	allLoaded.value = false;
  scrollToBottom();
  messageInput.value?.focus();
});

watch(
  () => messenger.messagesLoading,
  (loading) => {
    if (!loading && messenger.activeMessages.length > 0) {
      scrollToBottom();
    }
  }
);

watch(
  () => messenger.activeMessages.length,
  (newVal, oldVal) => {
    // Scroll if it's the first batch or if a new message is added to the end
    if (oldVal === 0 || (newVal > oldVal && !messenger.messagesLoading)) {
      scrollToBottom();
    }
  }
);

watch(
	() => typingUsers.value.length,
	(newVal, oldVal) => {
    if (newVal > oldVal) scrollToBottom();
  }
);

function handleSend(text: string) {
	messenger.sendMessage(text);
}
</script>
