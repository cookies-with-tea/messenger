<template>
  <aside class="flex flex-col h-full bg-abyss border-r border-border">
    <!-- Header -->
    <div class="flex items-center gap-3 px-4 py-4 border-b border-border">
      <span class="text-xl font-syne font-extrabold text-text-bright tracking-tight flex-1">pulsar</span>
      <span class="text-pulse text-xl mr-1">·</span>
      <!-- New chat button -->
      <button
        @click="modalOpen = true"
        class="p-1.5 rounded-lg text-text-dim hover:text-pulse hover:bg-pulse/10 border border-transparent hover:border-pulse/20 transition-all"
        title="New chat"
      >
        <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
          <path d="M10.75 4.75a.75.75 0 00-1.5 0v4.5h-4.5a.75.75 0 000 1.5h4.5v4.5a.75.75 0 001.5 0v-4.5h4.5a.75.75 0 000-1.5h-4.5v-4.5z"/>
        </svg>
      </button>
    </div>
    <NewChatModal :open="modalOpen" @close="modalOpen = false" />

    <!-- Search -->
    <div class="px-3 py-3 border-b border-border/50">
      <div class="relative">
        <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M9 3.5a5.5 5.5 0 100 11 5.5 5.5 0 000-11zM2 9a7 7 0 1112.452 4.391l3.328 3.329a.75.75 0 11-1.06 1.06l-3.329-3.328A7 7 0 012 9z" clip-rule="evenodd"/>
        </svg>
        <input
          v-model="search"
          type="text"
          placeholder="Search..."
          class="w-full bg-surface border border-border rounded-lg py-2 pl-8 pr-3 text-sm text-text-base placeholder:text-muted outline-none focus:border-pulse/50 focus:bg-elevated transition-all font-mono"
        />
      </div>
    </div>

    <!-- Chat list -->
    <div class="flex-1 overflow-y-auto py-2 px-2 space-y-1 scrollbar-thin">
      <div v-if="store.chatsLoading" class="flex justify-center py-12">
        <span class="text-xs font-mono text-muted animate-pulse">Loading chats...</span>
      </div>
      <template v-else-if="filteredChats.length">
        <ChatListItem
          v-for="chat in filteredChats"
          :key="chat.uuid"
          :chat="chat"
          :active="chat.uuid === store.activeChatId"
          @select="selectChat(chat.uuid)"
        />
      </template>
      <div v-else class="flex flex-col items-center gap-2 py-12 text-muted">
        <span class="text-2xl">🔍</span>
        <span class="text-sm font-mono">No chats found</span>
      </div>
    </div>

    <!-- User Profile & Logout -->
    <div class="p-3 border-t border-border bg-surface/30">
      <div class="flex items-center gap-3 p-2 rounded-xl bg-elevated/50 border border-border/50 group">
        <div class="relative">
          <div class="w-9 h-9 rounded-lg bg-gradient-to-br from-pulse/20 to-pulse/5 flex items-center justify-center border border-pulse/20 text-pulse font-bold text-sm shadow-[0_0_15px_rgba(var(--color-pulse),0.1)]">
            {{ store.currentUserId.slice(0, 2).toUpperCase() }}
          </div>
          <div class="absolute -bottom-0.5 -right-0.5 w-2.5 h-2.5 bg-pulse rounded-full border-2 border-abyss shadow-[0_0_8px_rgba(var(--color-pulse),0.5)]"></div>
        </div>
        
        <div class="flex-1 min-w-0">
          <div class="text-xs font-mono text-muted truncate">Logged in as</div>
          <div class="text-xs font-mono text-text-base truncate font-bold">{{ store.currentUserId.split('-')[0] }}...</div>
        </div>

        <button
          @click="handleLogout"
          class="p-2 rounded-lg text-text-dim hover:text-ember hover:bg-ember/10 border border-transparent hover:border-ember/20 transition-all group-hover:opacity-100 opacity-60 flex items-center justify-center"
          title="Logout"
        >
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4" />
            <polyline points="16 17 21 12 16 7" />
            <line x1="21" y1="12" x2="9" y2="12" />
          </svg>
        </button>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useMessengerStore } from '@/stores/messengerStore'
import ChatListItem from './ChatListItem.vue'
import NewChatModal from './NewChatModal.vue'
import { useRouter } from 'vue-router'

const modalOpen = ref(false)
const router = useRouter();

const store  = useMessengerStore()
const search = ref('')

const filteredChats = computed(() => {
  const q = search.value.toLowerCase().trim()
  if (!q) return store.chats
  return store.chats.filter(c =>
    (c.name ?? '').toLowerCase().includes(q) ||
    (c.last_message_body ?? '').toLowerCase().includes(q)
  )
})

const selectChat = (uuid: string) => {
	store.selectChat(uuid)

	router.push(`/chat/${uuid}`);
}

const handleLogout = () => {
  if (confirm('Are you sure you want to logout?')) {
    store.logout();
  }
}
</script>
