<script setup lang="ts">
import { ref, computed } from 'vue'
import { useMessengerStore } from '@/stores/messengerStore'
import ChatListItem from './ChatListItem.vue'
import NewChatModal from './NewChatModal.vue'
import BaseModal from './ui/BaseModal.vue'
import { useRouter } from 'vue-router'

const messenger = useMessengerStore()
const router = useRouter()

const modalOpen = ref(false)
const logoutModalOpen = ref(false)
const search = ref('')

const filteredChats = computed(() => {
  const q = search.value.toLowerCase().trim()
  if (!q) return messenger.chats
  return messenger.chats.filter(c =>
    (c.name ?? '').toLowerCase().includes(q) ||
    (c.last_message_body ?? '').toLowerCase().includes(q)
  )
})

const selectChat = (uuid: string) => {
	messenger.selectChat(uuid)

	if (router.currentRoute.value.path !== `/chat/${uuid}`) {
		router.push(`/chat/${uuid}`);
	}
}

const confirmLogout = () => {
  messenger.logout()
  logoutModalOpen.value = false
}
</script>

<template>
  <aside class="flex flex-col h-full glass border-r border-white/10 w-full relative z-10">
    <!-- Header -->
    <div class="flex items-center gap-3 px-4 sm:px-5 py-4 sm:py-5 border-b border-white/5 bg-white/2">
      <div class="flex items-center gap-2 flex-1">
        <div class="w-1.5 h-1.5 sm:w-2 sm:h-2 rounded-full bg-pulse shadow-[0_0_8px_rgba(var(--color-pulse),1)] animate-pulse"></div>
        <span class="text-lg sm:text-xl font-syne font-black text-text-bright tracking-widest uppercase">pulsar</span>
      </div>
      
      <!-- New chat button -->
      <button
        @click="modalOpen = true"
        class="p-1.5 sm:p-2 rounded-xl glass hover:bg-pulse/10 border border-white/5 hover:border-pulse/30 transition-all text-text-dim hover:text-pulse"
        title="New conversation"
      >
        <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
          <path d="M10.75 4.75a.75.75 0 00-1.5 0v4.5h-4.5a.75.75 0 000 1.5h4.5v4.5a.75.75 0 001.5 0v-4.5h4.5a.75.75 0 000-1.5h-4.5v-4.5z"/>
        </svg>
      </button>
    </div>
    <NewChatModal :open="modalOpen" @close="modalOpen = false" />

    <!-- Search & Quick Actions -->
    <div class="px-3 sm:px-4 py-3 sm:py-4 border-b border-white/5">
      <div class="relative group">
        <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted group-focus-within:text-pulse transition-colors" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M9 3.5a5.5 5.5 0 100 11 5.5 5.5 0 000-11zM2 9a7 7 0 1112.452 4.391l3.328 3.329a.75.75 0 11-1.06 1.06l-3.329-3.328A7 7 0 012 9z" clip-rule="evenodd"/>
        </svg>
        <input
          v-model="search"
          type="text"
          placeholder="Search..."
          class="w-full bg-white/3 border border-white/5 rounded-xl py-2 pl-9 pr-3 text-xs sm:text-sm text-text-base placeholder:text-muted outline-none focus:border-pulse/40 focus:bg-white/6 transition-all font-mono"
        />
      </div>
    </div>

    <!-- Chat list -->
    <div class="flex-1 overflow-y-auto py-2 px-1.5 sm:px-2 space-y-0.5 sm:space-y-1 scrollbar-thin">
      <div v-if="messenger.chatsLoading" class="flex justify-center py-12">
        <span class="text-xs font-mono text-muted animate-pulse">Scanning...</span>
      </div>
      <template v-else-if="filteredChats.length">
        <ChatListItem
          v-for="chat in filteredChats"
          :key="chat.uuid"
          :chat="chat"
          :active="chat.uuid === messenger.activeChatId"
          @select="selectChat(chat.uuid)"
        />
      </template>
      <div v-else class="flex flex-col items-center gap-2 py-12 text-muted">
        <span class="text-xl sm:text-2xl opacity-20">📡</span>
        <span class="text-[10px] sm:text-xs font-mono uppercase tracking-widest">No signals</span>
      </div>
    </div>

    <!-- User Profile & Logout -->
    <div class="p-3 sm:p-4 border-t border-white/5 bg-white/1">
      <div class="flex items-center gap-2.5 sm:gap-3 p-2 sm:p-2.5 rounded-2xl glass border border-white/10 group hover:border-white/20 transition-all backdrop-blur-xl">
        <div class="relative shrink-0">
          <div v-if="messenger.currentUserProfile?.avatar" class="w-8 h-8 sm:w-10 sm:h-10 rounded-xl overflow-hidden border border-white/10 bg-white/5 flex items-center justify-center group-hover:scale-105 transition-transform">
            <img :src="messenger.currentUserProfile.avatar" class="w-full h-full object-cover" :alt="messenger.currentUserProfile.first_name || 'User'" />
          </div>
          <div v-else class="w-8 h-8 sm:w-10 sm:h-10 rounded-xl bg-linear-to-br from-pulse/30 to-pulse/5 flex items-center justify-center border border-pulse/30 text-pulse font-bold text-xs sm:text-sm shadow-[0_0_20px_rgba(var(--color-pulse),0.1)] group-hover:scale-105 transition-transform">
            {{ (messenger.currentUserProfile?.first_name || 'U').slice(0, 1).toUpperCase() }}
          </div>
          <div class="absolute -bottom-0.5 -right-0.5 w-2.5 h-2.5 sm:w-3 sm:h-3 bg-sage rounded-full border-2 sm:border-3 border-abyss shadow-[0_0_10px_rgba(var(--color-sage),0.5)]"></div>
        </div>
        
        <div class="flex-1 min-w-0">
          <div class="text-[8px] sm:text-[9px] font-mono text-muted uppercase tracking-wider mb-0.5">Host</div>
          <div class="text-[11px] sm:text-xs font-mono text-text-bright truncate font-bold uppercase tracking-tight">
            {{ messenger.currentUserProfile ? `${messenger.currentUserProfile.first_name || ''} ${messenger.currentUserProfile.second_name || ''}`.trim() : 'Scanning...' }}
          </div>
        </div>

        <button
          @click="logoutModalOpen = true"
          class="p-2 sm:p-2.5 rounded-xl text-text-dim hover:text-ember hover:bg-ember/10 border border-transparent hover:border-ember/20 transition-all flex items-center justify-center group/btn"
          title="Exit Sector"
        >
          <svg class="w-3.5 h-3.5 sm:w-4 sm:h-4 group-hover/btn:translate-x-0.5 transition-transform" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4" />
            <polyline points="16 17 21 12 16 7" />
            <line x1="21" y1="12" x2="9" y2="12" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Reusable BaseModal for Logout -->
    <BaseModal 
      :show="logoutModalOpen" 
      title="Exit Sector?" 
      @close="logoutModalOpen = false"
    >
      <div class="space-y-4">
        <p class="text-xs sm:text-sm text-text-dim leading-relaxed">
          Are you sure you want to disconnect? Your current session will be terminated.
        </p>
      </div>
      <template #footer>
        <button 
          @click="logoutModalOpen = false"
          class="px-4 py-2 rounded-xl text-[10px] sm:text-xs font-mono text-muted hover:bg-white/5 transition-colors"
        >
          STAY
        </button>
        <button 
          @click="confirmLogout"
          class="px-4 sm:px-5 py-2 rounded-xl text-[10px] sm:text-xs font-mono bg-ember/10 text-ember hover:bg-ember/20 border border-ember/20 transition-all"
        >
          EXIT
        </button>
      </template>
    </BaseModal>
  </aside>
</template>
