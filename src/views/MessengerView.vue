<template>
  <div class="flex h-screen overflow-hidden bg-void text-text-base font-syne antialiased">
    <div class="w-72 shrink-0 flex flex-col overflow-hidden">
      <ChatSidebar />
    </div>
    <main class="flex-1 overflow-hidden flex flex-col">
      <ChatWindow v-if="store.activeChat" />
      <EmptyState v-else />
    </main>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useMessengerStore } from '@/stores/messengerStore'
import ChatSidebar from '@/components/ChatSidebar.vue'
import ChatWindow  from '@/components/ChatWindow.vue'
import EmptyState  from '@/components/EmptyState.vue'

const store = useMessengerStore()

onMounted(() => store.fetchChats())
</script>
