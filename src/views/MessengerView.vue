<template>
  <div class="flex h-screen overflow-hidden bg-void text-text-base font-syne antialiased relative">
    <!-- Cosmic Background Layer -->
    <div class="starfield-container">
      <div class="stars-1"></div>
      <div class="stars-2"></div>
      <div class="stars-3"></div>
    </div>

    <!-- Sidebar: Hidden on mobile if a chat is active -->
    <div 
      class="w-full md:w-80 shrink-0 flex flex-col overflow-hidden border-r border-border"
      :class="{ 'hidden md:flex': store.activeChatId }"
    >
      <ChatSidebar />
    </div>

    <!-- Main Content: Hidden on mobile if no chat is active -->
    <main 
      class="flex-1 overflow-hidden flex flex-col"
      :class="{ 'hidden md:flex': !store.activeChatId }"
    >
      <ChatWindow v-if="store.activeChat" />
      <EmptyState v-else class="hidden md:flex" />
    </main>
    
    <!-- WebRTC Call Modal Overlay -->
    <CallModal />

    <!-- User Profile Modal Overlay -->
    <UserProfileModal @open-edit="isEditProfileOpen = true" />

    <!-- Edit Profile Modal Overlay -->
    <EditProfileModal 
      :is-open="isEditProfileOpen" 
      :user="store.currentUserProfile"
      @close="isEditProfileOpen = false"
    />
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useMessengerStore } from '@/stores/messengerStore'
import ChatSidebar from '@/components/ChatSidebar.vue'
import ChatWindow  from '@/components/ChatWindow.vue'
import EmptyState  from '@/components/EmptyState.vue'
import CallModal from '@/components/CallModal.vue'
import UserProfileModal from '@/components/UserProfileModal.vue'
import EditProfileModal from '@/components/EditProfileModal.vue'
import { ref } from 'vue'

const store = useMessengerStore()
const isEditProfileOpen = ref(false)

onMounted(() => {
  // Открываем глобальный WS-канал если ещё не открыт (например, после refresh страницы)
  store.initWs()
  store.fetchChats()
})
</script>
